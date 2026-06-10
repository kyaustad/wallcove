/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly PUBLIC_DEMO_IMAGE_1: string;
  readonly PUBLIC_DEMO_IMAGE_2: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
