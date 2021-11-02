import getConfig from "next/config";

const { publicRuntimeConfig, serverRuntimeConfig } = getConfig();

export const {
  TEMPLATE_VERSION,
  TEMPLATE_API_URL,
  TEMPLATE_API_PUBLIC_URL,
  SENTRY_DSN,
} = {
  ...publicRuntimeConfig,
  ...serverRuntimeConfig,
} as any;
