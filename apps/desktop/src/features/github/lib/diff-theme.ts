import { registerCustomTheme, resolveTheme } from "@pierre/diffs";

const faint = "#636363";

const shades = { dark: "#a3a3a3", light: "#525252" } as const;

export const diffThemes = {
  dark: "fogde-dark",
  light: "fogde-light",
} as const;

interface Rule {
  settings?: { foreground?: string };
}

function brighten<T extends Rule>(rules: T[] | undefined, shade: string) {
  return rules?.map((rule) =>
    rule.settings?.foreground?.toLowerCase() === faint
      ? { ...rule, settings: { ...rule.settings, foreground: shade } }
      : rule,
  );
}

for (const type of ["dark", "light"] as const) {
  registerCustomTheme(diffThemes[type], async () => {
    const { textmate } = await resolveTheme(`pierre-${type}`);
    if (!textmate) {
      throw new Error(`The pierre-${type} diff theme has no colours`);
    }

    return {
      ...textmate,
      name: diffThemes[type],
      settings: brighten(textmate.settings, shades[type]) ?? [],
      tokenColors: brighten(textmate.tokenColors, shades[type]),
    };
  });
}
