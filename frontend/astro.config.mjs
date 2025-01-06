// @ts-check
import { defineConfig } from "astro/config";
import react from '@astrojs/react';

import tailwind from '@astrojs/tailwind';

// https://astro.build/config
export default defineConfig({
  server: {
    port: Number(process.env.PORT || 3000)
  },
  vite: {
    ssr: {
      noExternal: ['effector', 'effector-react']
    }
  },
  integrations: [react(), tailwind()]
});