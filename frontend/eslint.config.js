import eslintPluginAstro from "eslint-plugin-astro";
import tseslint from "typescript-eslint";
/* 
export default [
  // add more generic rule sets here, such as:
  // js.configs.recommended,
  ...eslintPluginAstro.configs.recommended,
  {
    rules: {
      quotes: ["error", "double"],
      semi: ["error", "always"],
      "comma-dangle": ["error", "never"]
    }
  }
];
*/
export default tseslint.config(
  {
    ignores: ["**/dist", "**/node_modules", "**/.astro", "**/.github", "**/.changeset"]
  },
  // TypeScript
  ...tseslint.configs.recommended,
  {
    rules: {
      "@typescript-eslint/no-explicit-any": "off"
    }
  },
  // Allow triple-slash references in `*.d.ts` files.
  {
    files: ["**/*.d.ts"],
    rules: {
      "@typescript-eslint/triple-slash-reference": "off"
    }
  },
  // Astro
  ...[
    // add more generic rule sets here, such as:
    // js.configs.recommended,
    ...eslintPluginAstro.configs.recommended,
    {
      rules: {
        quotes: ["error", "double"],
        semi: ["error", "always"],
        "comma-dangle": ["error", "never"]
      }
    }
  ]
);