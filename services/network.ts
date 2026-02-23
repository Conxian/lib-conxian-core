/**
 * Conxian Gateway Network Routing
 * Replaces legacy Anya-core and OPSource endpoints.
 */

/**
 * Returns the correct Gateway API route.
 *
 * @param service - The name of the service (e.g., 'bisq', 'rgb')
 * @param environment - The execution environment ('local', 'production', etc.)
 */
export const getGatewayUrl = (service: string, environment: string): string => {
  // Support seamless switching between local execution and production execution
  const isProduction = environment === 'production';
  const baseUrl = isProduction
    ? 'https://gateway.conxian.com'
    : 'http://localhost:8080';

  // Returns the correct Gateway API route (/api/v1/...)
  return `${baseUrl}/api/v1/${service}`;
};

// Current environment selection logic
const currentEnv = process.env.NODE_ENV === 'production' ? 'production' : 'local';

// Sovereign Service Base URLs refactored to resolve through getGatewayUrl
export const BISQ_API_URL = getGatewayUrl('bisq', currentEnv);
export const RGB_API_URL = getGatewayUrl('rgb', currentEnv);
export const BITVM_API_URL = getGatewayUrl('bitvm', currentEnv);
export const CHANGELLY_API_URL = getGatewayUrl('changelly', currentEnv);

// Bitcoin Layer 2 & Sidechain URLs
export const STACKS_API_URL = getGatewayUrl('stacks', currentEnv);
export const LIGHTNING_API_URL = getGatewayUrl('lightning', currentEnv);
export const LIQUID_API_URL = getGatewayUrl('liquid', currentEnv);
export const ROOTSTOCK_API_URL = getGatewayUrl('rootstock', currentEnv);
export const BABYLON_API_URL = getGatewayUrl('babylon', currentEnv);
export const BOB_API_URL = getGatewayUrl('bob', currentEnv);
export const MERLIN_API_URL = getGatewayUrl('merlin', currentEnv);
export const BOTANIX_API_URL = getGatewayUrl('botanix', currentEnv);
export const B2NETWORK_API_URL = getGatewayUrl('b2network', currentEnv);
export const CITREA_API_URL = getGatewayUrl('citrea', currentEnv);
export const BITLAYER_API_URL = getGatewayUrl('bitlayer', currentEnv);
export const ALPEN_API_URL = getGatewayUrl('alpen', currentEnv);
export const MEZO_API_URL = getGatewayUrl('mezo', currentEnv);
export const ZULU_API_URL = getGatewayUrl('zulu', currentEnv);
export const BISON_API_URL = getGatewayUrl('bison', currentEnv);
export const HEMI_API_URL = getGatewayUrl('hemi', currentEnv);
export const TAPROOT_ASSETS_API_URL = getGatewayUrl('taproot-assets', currentEnv);
export const NUBIT_API_URL = getGatewayUrl('nubit', currentEnv);
export const LORENZO_API_URL = getGatewayUrl('lorenzo', currentEnv);
export const LAYERS_API_URL = getGatewayUrl('layers', currentEnv);

// System & Monitoring Endpoints
export const HEALTH_API_URL = getGatewayUrl('health', currentEnv);
export const STATUS_API_URL = getGatewayUrl('status', currentEnv);
export const COMPLIANCE_API_URL = getGatewayUrl('compliance', currentEnv);
export const METRICS_API_URL = getGatewayUrl('metrics', currentEnv);
export const RESERVES_API_URL = getGatewayUrl('reserves', currentEnv);
export const PRICES_API_URL = getGatewayUrl('prices', currentEnv);

/**
 * Phase 4: Protocol-Specific Helpers
 */
export const createLightningInvoice = async (amountMsat: number, description: string) => {
  const url = `${getGatewayUrl("lightning", currentEnv)}/invoice`;
  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ amount_msat: amountMsat, description }),
  });
  return response.json();
};

export const payLightningInvoice = async (invoice: string) => {
  const url = `${getGatewayUrl("lightning", currentEnv)}/pay`;
  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ invoice }),
  });
  return response.json();
};

export const getStacksContract = async (contractId: string) => {
  const url = `${getGatewayUrl("stacks", currentEnv)}/contract/${contractId}`;
  const response = await fetch(url);
  return response.json();
};
export const getRgbContract = async (contractId: string) => {
  const url = `${getGatewayUrl("rgb", currentEnv)}/contract/${contractId}`;
  const response = await fetch(url);
  return response.json();
};

export const getBitvmProof = async (proofId: string) => {
  const url = `${getGatewayUrl("bitvm", currentEnv)}/proof/${proofId}`;
  const response = await fetch(url);
  return response.json();
};

export const getPrices = async () => {
  const url = getGatewayUrl("prices", currentEnv);
  const response = await fetch(url);
  return response.json();
};

export const getExchangeRate = async (from: string, to: string) => {
  const url = `${getGatewayUrl("changelly", currentEnv)}/rate?from=${from}&to=${to}`;
  const response = await fetch(url);
  return response.json();
};

export const getTaprootAssets = async () => {
  const url = getGatewayUrl("taproot-assets", currentEnv);
  const response = await fetch(url);
  return response.json();
};
export const checkCompliance = async (address: string) => {
  const url = `${getGatewayUrl("compliance", currentEnv)}/check`;
  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ address }),
  });
  return response.json();
};