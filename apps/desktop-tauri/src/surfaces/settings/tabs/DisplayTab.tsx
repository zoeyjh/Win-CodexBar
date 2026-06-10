import { useLocale } from "../../../hooks/useLocale";
import { Field, Toggle } from "../../../components/FormControls";
import type { TabProps } from "../../Settings";
import { FloatBarSettingsSection } from "../../../floatbar";

export default function DisplayTab({ settings, set, saving }: TabProps) {
  const { t } = useLocale();
  return (
    <>
      {/* ── Menu content ─────────────────────────────────────────── */}
      <section className="settings-section">
        <h3 className="settings-section__title">Menu Content</h3>
        <div className="settings-section__group">
          <Field
            label={t("ShowAsUsedLabel")}
            description={t("ShowAsUsedHelper")}
            leading
          >
            <Toggle
              checked={settings.showAsUsed}
              disabled={saving}
              onChange={(v) => set({ showAsUsed: v })}
            />
          </Field>
          <Field
            label={t("ShowCreditsExtra")}
            description={t("ShowCreditsExtraHelper")}
            leading
          >
            <Toggle
              checked={settings.showCreditsExtraUsage}
              disabled={saving}
              onChange={(v) => set({ showCreditsExtraUsage: v })}
            />
          </Field>
          <Field
            label={t("ShowAllTokenAccountsLabel")}
            description={t("ShowAllTokenAccountsHelper")}
            leading
          >
            <Toggle
              checked={settings.showAllTokenAccountsInMenu}
              disabled={saving}
              onChange={(v) => set({ showAllTokenAccountsInMenu: v })}
            />
          </Field>
          <Field
            label={t("ResetTimeRelative")}
            description={t("ResetTimeRelativeHelper")}
            leading
          >
            <Toggle
              checked={settings.resetTimeRelative}
              disabled={saving}
              onChange={(v) => set({ resetTimeRelative: v })}
            />
          </Field>
        </div>
      </section>

      <FloatBarSettingsSection settings={settings} saving={saving} set={set} />
    </>
  );
}
