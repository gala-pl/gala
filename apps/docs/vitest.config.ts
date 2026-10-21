import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "node",
    include: ["**/*.test.ts"],
    typecheck: {
      enabled: true,
      checker: "tsc",
      tsconfig: "./tsconfig.test.json",
    },
  },
});
