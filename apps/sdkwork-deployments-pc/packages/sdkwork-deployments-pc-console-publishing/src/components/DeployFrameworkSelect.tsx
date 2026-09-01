/**
 * Framework & architecture select (v3.2: 目录步骤内、路径下方).
 *
 * 参照 Vercel/Railway 导入流程：根据所选目录自动检测框架（目录标记如
 * `unpackage`/`.dart_tool`/`src-tauri`/`.nuxt`），命中项带「检测到」徽标；
 * 检测不到时回退卡片默认框架，用户始终可手动覆盖。每个选项预览其默认
 * 构建产物目录。
 */
import type { PublishingTranslator } from "../i18n.ts";
import {
  type DeployFrameworkOption,
} from "../service/deploy-app-publishing.ts";
import css from "./create-deploy-app.module.css";

export interface DeployFrameworkSelectProps {
  /** Framework options of the selected card. */
  readonly frameworks: readonly DeployFrameworkOption[]
  /** Selected framework id. */
  readonly frameworkId: string | undefined
  /** Framework id auto-detected from the directory listing, if any. */
  readonly autoDetectedId?: string | undefined
  readonly t: PublishingTranslator
  readonly onChange: (frameworkId: string) => void
}

export function DeployFrameworkSelect({ frameworks, frameworkId, autoDetectedId, t, onChange }: DeployFrameworkSelectProps) {
  return (
    <div className={css.field}>
      <span className={css.fieldLabel}>{t("frameworkLabel")}</span>
      <span className={css.fieldHint}>{t("frameworkStepHint")}</span>
      <div className={css.frameworkGrid} role="radiogroup" aria-label={t("frameworkLabel")}>
        {frameworks.map((framework) => {
          const selected = frameworkId === framework.id;
          const autoDetected = autoDetectedId === framework.id;
          return (
            <button
              key={framework.id}
              type="button"
              role="radio"
              aria-checked={selected}
              data-selected={selected}
              className={css.frameworkTile}
              onClick={() => { onChange(framework.id) }}
            >
              <span className={css.frameworkName}>{t(framework.labelKey)}</span>
              {framework.buildOutputPath !== undefined && (
                <span className={css.frameworkBuildOutput}>
                  {t("frameworkBuildOutputPrefix")}<code>{framework.buildOutputPath}</code>
                </span>
              )}
              {autoDetected && <span className={css.typeCardBadge}>{t("typeSuggested")}</span>}
            </button>
          );
        })}
      </div>
    </div>
  );
}
