import { useEffect, useRef, useState } from "react";
import type { DeploymentsLocale } from "@sdkwork/deployments-pc-commons";
import { publishingTranslator } from "../i18n.ts";
import css from "./create-deploy-app.module.css";

/**
 * 构建输出流帧（线协议镜像）。与宿主插件的 DeployHostBuildFrame 保持结构
 * 一致：本包是宿主无关能力包，不反向依赖宿主类型，端口由宿主适配注入。
 */
export type DeployDialogBuildFrame =
  | { readonly type: "started"; readonly buildId: string; readonly command: string; readonly cwd: string }
  | { readonly type: "output"; readonly buildId: string; readonly stream: "stdout" | "stderr"; readonly text: string }
  | {
    readonly type: "exit"
    readonly buildId: string
    readonly outcome: "succeeded" | "failed" | "cancelled"
    readonly exitCode: number | null
    readonly signal: string | null
    readonly durationMs: number
  };

/**
 * 宿主构建端口（结构最小接口）：一键打包由宿主插件绑定 sdkworkAppBuild
 * Remote 后注入；端口缺席时发布对话框隐藏打包入口而非报错。
 */
export interface DeployDialogBuildPort {
  start(request: { cwd: string; script?: string }): Promise<{ buildId: string; command: string; cwd: string }>;
  /** 消费一次构建的输出帧直到 exit 帧；resolve 即流结束。 */
  follow(
    buildId: string,
    onFrame: (frame: DeployDialogBuildFrame) => void,
    signal: AbortSignal,
  ): Promise<void>;
  cancel(buildId: string): Promise<void>;
}

/** 构建阶段。 */
type BuildPhase = "starting" | "running" | "exited";

export interface BuildProgressDialogProps {
  readonly locale: DeploymentsLocale
  /** 主题（驱动组件内建 CSS 变量切换；缺省浅色）。 */
  readonly theme?: ("light" | "dark") | undefined
  /** 打包目录（发布对话框当前源码目录 / 规范表面根）。 */
  readonly cwd: string
  /** 构建脚本名（缺省 build）。 */
  readonly script?: string | undefined
  readonly port: DeployDialogBuildPort
  readonly onClose: () => void
  /** 构建进程退出时回调（含取消）；宿主据此决定是否复检产物目录。 */
  readonly onFinished?: ((outcome: "succeeded" | "failed" | "cancelled") => void) | undefined
}

/**
 * 一键打包进度弹窗：展示打包命令、实时输出（stdout/stderr 着色区分）与
 * 退出结果。构建进程由宿主 sdkwork-app-build 插件执行，弹窗只做呈现与
 * 取消转发；关闭弹窗仅断开跟随，不终止构建。
 */
export function BuildProgressDialog({
  locale,
  theme = "light",
  cwd,
  script,
  port,
  onClose,
  onFinished,
}: BuildProgressDialogProps) {
  const t = publishingTranslator(locale)
  const [phase, setPhase] = useState<BuildPhase>("starting")
  const [command, setCommand] = useState<string>()
  const [buildId, setBuildId] = useState<string>()
  const [lines, setLines] = useState<{ stream: "stdout" | "stderr"; text: string }[]>([])
  const [outcome, setOutcome] = useState<"succeeded" | "failed" | "cancelled">()
  const [exitCode, setExitCode] = useState<number | null>()
  const [durationMs, setDurationMs] = useState<number>()
  const [failure, setFailure] = useState<string>()
  const logRef = useRef<HTMLDivElement | null>(null)
  // StrictMode 双挂载守卫：start 只能发起一次，重复 start 会撞并发上限。
  const startedRef = useRef(false)

  useEffect(() => {
    if (startedRef.current) return
    startedRef.current = true
    const controller = new AbortController()
    let disposed = false
    void (async () => {
      try {
        const started = await port.start(script === undefined ? { cwd } : { cwd, script })
        if (disposed) return
        setBuildId(started.buildId)
        setCommand(started.command)
        setPhase("running")
        await port.follow(started.buildId, (frame) => {
          if (frame.type === "started") {
            setCommand(frame.command)
            return
          }
          if (frame.type === "output") {
            setLines((current) => [...current, { stream: frame.stream, text: frame.text }])
            return
          }
          setOutcome(frame.outcome)
          setExitCode(frame.exitCode)
          setDurationMs(frame.durationMs)
          setPhase("exited")
          onFinished?.(frame.outcome)
        }, controller.signal)
      } catch (cause) {
        if (disposed) return
        setFailure(cause instanceof Error ? cause.message : String(cause))
        setPhase("exited")
      }
    })()
    return () => {
      disposed = true
      controller.abort()
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps -- 端口与目标目录随挂载固定
  }, [])

  // 输出追加后自动滚动到底部（用户上滚查看历史时不打断）。
  useEffect(() => {
    const log = logRef.current
    if (log === null) return
    log.scrollTop = log.scrollHeight
  }, [lines])

  const cancelBuild = () => {
    if (buildId === undefined) return
    void port.cancel(buildId).catch(() => {
      // 取消转发失败不打断展示：exit 帧仍会到达（进程自然结束）。
    })
  }

  const statusText = (): string => {
    if (failure !== undefined) return t("buildStartFailed", { message: failure })
    if (phase === "starting") return t("buildStarting")
    if (phase === "running") return t("buildRunning")
    if (outcome === "succeeded") return t("buildSucceeded", { seconds: formatSeconds(durationMs) })
    if (outcome === "failed") return t("buildFailed", { code: String(exitCode ?? "?") })
    return t("buildCancelled")
  }

  return (
    <div className={css.publishDialog} data-theme={theme} role="presentation">
      <div className={`${css.dialog} ${css.buildDialog}`} role="dialog" aria-modal="true" aria-label={t("buildDialogTitle")}>
        <header className={css.header}>
          <div className={css.headerText}>
            <h2>{t("buildDialogTitle")}</h2>
            <p>{t("buildDialogDescription", { cwd })}</p>
          </div>
          <button type="button" className={css.closeButton} title={t("close")} aria-label={t("close")} onClick={onClose}>
            ×
          </button>
        </header>

        <div className={css.body}>
          <div className={css.buildMeta}>
            <span className={css.fieldLabel}>{t("buildCommand")}</span>
            <code className={css.buildCommand}>{command ?? t("buildPendingCommand")}</code>
          </div>
          <div className={css.buildLog} ref={logRef} aria-live="polite">
            {lines.map((line, index) => (
              <div key={index} className={line.stream === "stderr" ? css.buildLineStderr : css.buildLine}>
                {line.text}
              </div>
            ))}
            {lines.length === 0 && phase !== "exited" && <div className={css.buildLine}>{t("buildNoOutputYet")}</div>}
          </div>
        </div>

        <footer className={css.footer}>
          <div
            className={`${css.buildStatus} ${
              outcome === "succeeded" ? css.buildStatusOk : outcome === "failed" || failure !== undefined ? css.buildStatusBad : ""
            }`}
            role="status"
          >
            {statusText()}
          </div>
          {phase === "running" && (
            <button type="button" className={css.secondaryButton} onClick={cancelBuild}>
              {t("buildCancel")}
            </button>
          )}
          <button
            type="button"
            className={css.primaryButton}
            onClick={onClose}
          >
            {outcome === "succeeded" ? t("buildDone") : t("close")}
          </button>
        </footer>
      </div>
    </div>
  )
}

/** 毫秒 → "12.3s" 展示。 */
function formatSeconds(durationMs: number | undefined): string {
  if (durationMs === undefined || durationMs < 0) return "-"
  return `${(durationMs / 1000).toFixed(1)}s`
}
