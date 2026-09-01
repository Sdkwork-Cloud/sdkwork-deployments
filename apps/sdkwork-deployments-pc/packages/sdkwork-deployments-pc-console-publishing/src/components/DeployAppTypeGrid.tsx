/**
 * Application-type grid (发布对话框 v3 第 1 步).
 *
 * Icon + name cards per the product requirement: the dialog opens on the
 * type grid. Framework / architecture selection (native, Flutter, React
 * Native, …) deliberately lives in the *next* step, not inline here, so every
 * card stays one tap. Cards whose surface was auto-detected in the project
 * directory carry a "检测到" badge.
 */
import type { PublishingTranslator } from "../i18n.ts";
import {
  DEPLOY_APP_TYPE_CARDS,
  type DeployAppTypeCard,
} from "../service/deploy-app-publishing.ts";
import { DeployAppTypeIcon } from "./DeployAppTypeIcon.tsx";
import css from "./create-deploy-app.module.css";

export interface DeployAppTypeGridProps {
  /** Selected primary card id. */
  readonly cardId: string | undefined
  readonly onChange: (cardId: string) => void
  /** Card surfaced by directory auto-detection, if any. */
  readonly suggestedCardId?: string | undefined
  readonly t: PublishingTranslator
}

export function DeployAppTypeGrid({ cardId, onChange, suggestedCardId, t }: DeployAppTypeGridProps) {
  const selectCard = (card: DeployAppTypeCard) => {
    onChange(card.id);
  };

  return (
    <div className={css.field}>
      <span className={css.fieldLabel}>{t("appType")}</span>
      <span className={css.fieldHint}>{t("appTypeGridHint")}</span>
      <div className={css.typeCardGrid} role="radiogroup" aria-label={t("appType")}>
        {DEPLOY_APP_TYPE_CARDS.map((card) => {
          const selected = cardId === card.id;
          const suggested = suggestedCardId === card.id;
          return (
            <button
              key={card.id}
              type="button"
              role="radio"
              aria-checked={selected}
              data-selected={selected}
              className={css.typeCardTile}
              onClick={() => { selectCard(card) }}
            >
              <span className={css.typeCardIcon}><DeployAppTypeIcon iconKey={card.iconKey} /></span>
              <span className={css.typeCardName}>{t(card.labelKey)}</span>
              <span className={css.typeCardHint}>{t(card.hintKey)}</span>
              {suggested && <span className={css.typeCardBadge}>{t("typeSuggested")}</span>}
            </button>
          );
        })}
      </div>
    </div>
  );
}
