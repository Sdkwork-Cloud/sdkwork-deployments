/**
 * Multi-level category cascade selector (需求 3).
 *
 * Renders one `<select>` per level driven by the declarative taxonomy, filtered
 * by the selected app kind. Selecting a level-1 node repopulates level 2, and
 * so on. The final selection is emitted as `{ id, path }` and persisted into
 * `deploy_app.metadata.category`.
 */
import { useMemo, useState } from "react";
import type { AppKind } from "@sdkwork/deployments-app-sdk";
import type { PublishingTranslator } from "../i18n.ts";
import { categoriesForAppKind, findCategoryNode, type DeployAppCategoryNode } from "../service/app-categories.ts";
import type { DeployAppCategorySelection } from "../service/deploy-app-publishing.ts";
import css from "./create-deploy-app.module.css";

export interface CategoryCascadeSelectProps {
  readonly appKind: AppKind | undefined
  readonly value: DeployAppCategorySelection | undefined
  readonly onChange: (selection: DeployAppCategorySelection | undefined) => void
  readonly t: PublishingTranslator
}

/** Collect the selectable level list for one index (0-based, capped at 3). */
function optionsForLevel(
  level: number,
  selection: readonly string[],
  tree: readonly DeployAppCategoryNode[],
): readonly DeployAppCategoryNode[] {
  if (level === 0) return tree
  const parent = findCategoryNode(selection[level - 1] ?? "", tree)
  return parent?.children ?? []
}

export function CategoryCascadeSelect({ appKind, value, onChange, t }: CategoryCascadeSelectProps) {
  const tree = useMemo(() => categoriesForAppKind(appKind), [appKind])
  const [selection, setSelection] = useState<string[]>(() => value?.path.map((entry) => entry.id) ?? [])

  const levels = 3

  const selectAt = (level: number, id: string) => {
    const next = [...selection.slice(0, level), id]
    setSelection(next)
    const node = id ? findCategoryNode(id, tree) : undefined
    if (!node) {
      onChange(undefined)
      return
    }
    const path = node ? pathOf(node, tree) : []
    onChange({ id, path })
  }

  const clear = () => {
    setSelection([])
    onChange(undefined)
  }

  return (
    <div className={css.categoryRow}>
      {Array.from({ length: levels }, (_, index) => {
        const options = optionsForLevel(index, selection, tree)
        if (options.length === 0) {
          return (
            <select key={index} className={css.select} disabled aria-label={t(index === 0 ? "categoryLevel1" : index === 1 ? "categoryLevel2" : "categoryLevel3")}>
              <option>{t("categoryNone")}</option>
            </select>
          )
        }
        const current = selection[index] ?? ""
        return (
          <select
            key={index}
            className={css.select}
            value={current}
            aria-label={t(index === 0 ? "categoryLevel1" : index === 1 ? "categoryLevel2" : "categoryLevel3")}
            onChange={(event) => { selectAt(index, event.target.value) }}
          >
            <option value="">{t("categoryPlaceholder", { level: String(index + 1) })}</option>
            {options.map((node) => (
              <option key={node.id} value={node.id}>{t(node.labelKey)}</option>
            ))}
          </select>
        )
      })}
      <button type="button" className={`${css.secondaryButton} ${css.categoryClear}`} onClick={clear}>
        {t("categoryClear")}
      </button>
    </div>
  )
}

function pathOf(node: DeployAppCategoryNode, tree: readonly DeployAppCategoryNode[]): readonly { id: string; label: string }[] {
  const chain: { id: string; label: string }[] = []
  const walk = (candidates: readonly DeployAppCategoryNode[], target: DeployAppCategoryNode): boolean => {
    for (const candidate of candidates) {
      chain.push({ id: candidate.id, label: candidate.labelKey })
      if (candidate.id === target.id) return true
      if (candidate.children && walk(candidate.children, target)) return true
      chain.pop()
    }
    return false
  }
  walk(tree, node)
  return chain
}
