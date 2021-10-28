const { TEMPLATE_API_URL, TEMPLATE_API_PUBLIC_URL } = process.env;
const { GCP_API_KEY } = process.env;

/**
 * @type {import('next').NextConfig}
 **/
const config = {
  productionBrowserSourceMaps: true,
  publicRuntimeConfig: {
    GCP_API_KEY,
    TEMPLATE_API_PUBLIC_URL,
  },
  serverRuntimeConfig: {
    GCP_API_KEY,
    TEMPLATE_API_URL,
  },
};

module.exports = config;
