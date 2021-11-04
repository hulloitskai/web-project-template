const { withSentryConfig } = require("@sentry/nextjs");

const TEMPLATE_VERSION = process.env.npm_package_version;
const { TEMPLATE_API_URL, TEMPLATE_API_PUBLIC_URL } = process.env;

const {
  SENTRY_URL,
  SENTRY_ORG,
  SENTRY_PROJECT,
  SENTRY_DSN,
  SENTRY_AUTH_TOKEN,
} = process.env;

/** @type {import('next').NextConfig} */
const config = {
  publicRuntimeConfig: {
    TEMPLATE_VERSION,
    TEMPLATE_API_PUBLIC_URL,
    SENTRY_DSN,
  },
  serverRuntimeConfig: {
    TEMPLATE_VERSION,
    TEMPLATE_API_URL,
    SENTRY_DSN,
  },
};

if (!!SENTRY_URL && !!SENTRY_ORG && !!SENTRY_PROJECT && !!SENTRY_AUTH_TOKEN) {
  const { npm_package_name: packageName } = process.env;
  const { npm_package_name: packageVersion } = process.env;

  /** @type {import('@sentry/webpack-plugin').SentryCliPluginOptions} */
  const sentryOptions = {
    silent: true,
    release: `${packageName}@${packageVersion}`,
  };

  module.exports = withSentryConfig(config, sentryOptions);
} else {
  const missingVariables = Object.entries({
    SENTRY_URL,
    SENTRY_ORG,
    SENTRY_PROJECT,
    SENTRY_AUTH_TOKEN,
  })
    .filter(([, value]) => !value)
    .map(([key]) => key);

  console.warn(
    `[Sentry] Skip uploading sourcemaps (missing variables: ${missingVariables.join(
      ", ",
    )})`,
  );
  module.exports = config;
  module.exports = config;
}
