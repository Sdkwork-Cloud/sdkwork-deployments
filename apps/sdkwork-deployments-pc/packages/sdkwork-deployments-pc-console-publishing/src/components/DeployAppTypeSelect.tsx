/**
 * Application type selector (需求 2).
 *
 * Each card maps to one `deploy_app.app_kind` (+ platform target tech stack).
 * The selected option is carried into the publish request.
 */
import type { PublishingTranslator } from "../i18n.ts";
import { DEPLOY_APP_TYPE_OPTIONS, type DeployAppTypeOption } from "../service/deploy-app-publishing.ts";
import css from "./create-deploy-app.module.css";

export interface DeployAppTypeSelectProps {
  readonly value: DeployAppTypeOption | undefined
  readonly onChange: (option: DeployAppTypeOption) => void
  readonly t: PublishingTranslator
}

export function DeployAppTypeSelect({ value, onChange, t }: DeployAppTypeSelectProps) {
  return (
    <div className={css.typeGrid} role="radiogroup" aria-label={t("appType")}>
      {DEPLOY_APP_TYPE_OPTIONS.map((option) => (
        <button
          key={option.id}
          type="button"
          role="radio"
          aria-checked={value?.id === option.id}
          data-selected={value?.id === option.id}
          className={css.typeCard}
          onClick={() => { onChange(option) }}
        >
          <strong>{t(option.labelKey)}</strong>
          <small>{t(option.hintKey)}</small>
        </button>
      ))}
    </div>
  )
}
