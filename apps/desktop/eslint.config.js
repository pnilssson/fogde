import { readdirSync } from "node:fs";
import path from "node:path";

import js from "@eslint/js";
import { defineConfig, globalIgnores } from "eslint/config";
import reactHooks from "eslint-plugin-react-hooks";
import { reactRefresh } from "eslint-plugin-react-refresh";
import globals from "globals";
import tseslint from "typescript-eslint";

import { spacing } from "./lint/spacing.js";

const parent = {
  group: ["../*"],
  message: "Import with @/ instead of reaching into parent folders.",
};
const layers = {
  group: ["@/app/*", "@/routes/*"],
  message: "Only routes and the app may import routes or the app.",
};
const features = readdirSync(path.join(import.meta.dirname, "src/features"));

const restrict = (...patterns) => ({
  "no-restricted-imports": ["error", { patterns: [parent, ...patterns] }],
});

export default defineConfig([
  globalIgnores(["dist", "src-tauri", "src/generated"]),
  {
    files: ["**/*.{ts,tsx}"],
    extends: [
      js.configs.recommended,
      tseslint.configs.strictTypeChecked,
      tseslint.configs.stylisticTypeChecked,
      reactHooks.configs.flat.recommended,
      reactRefresh.configs.vite(),
    ],
    languageOptions: {
      globals: globals.browser,
      parserOptions: {
        projectService: true,
      },
    },
    rules: {
      ...restrict(),
      "@typescript-eslint/only-throw-error": [
        "error",
        {
          allow: [
            {
              from: "package",
              package: "@tanstack/router-core",
              name: "Redirect",
            },
          ],
        },
      ],
    },
  },
  {
    files: ["src/**/*.{ts,tsx}"],
    ignores: ["src/components/ui/**"],
    plugins: { fogde: { rules: { spacing } } },
    rules: { "fogde/spacing": "error" },
  },
  {
    files: ["src/components/ui/**"],
    rules: { "react-refresh/only-export-components": "off" },
  },
  {
    files: ["src/components/ui/sidebar.tsx"],
    rules: { "@typescript-eslint/consistent-type-definitions": "off" },
  },
  {
    files: ["src/components/ui/sidebar.tsx", "src/hooks/use-mobile.ts"],
    rules: {
      "@typescript-eslint/restrict-template-expressions": "off",
      "@typescript-eslint/no-confusing-void-expression": "off",
    },
  },
  {
    files: ["src/hooks/use-mobile.ts"],
    rules: { "react-hooks/set-state-in-effect": "off" },
  },
  {
    files: ["src/components/**", "src/hooks/**", "src/lib/**"],
    rules: restrict(layers, {
      group: ["@/features/*"],
      message: "Shared code cannot import features.",
    }),
  },
  ...features.map((feature) => ({
    files: [`src/features/${feature}/**`],
    rules: restrict(layers, {
      group: ["@/features/*", `!@/features/${feature}`],
      message:
        "A feature cannot import another feature; compose them in a route.",
    }),
  })),
]);
