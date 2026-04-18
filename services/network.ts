import fetch from 'node-fetch';

/**
 * Service and System Models
 */
export interface ServiceStatus {
  name: string;
  status: string;
  last_checked: string;
  latency_ms: number;
  trust_model: string;
  risk_level: string;
  data_availability: string;
  settlement: string;
  bridge_security: string;
  tvl_usd: number;
  metadata: Record<string, string>;
}

export interface FinancialMetrics {
  mrr_usd: number;
  arr_usd: number;
  churn_rate_pct: number;
  protocol_fees_collected_usd: number;
  last_updated: string;
}

export interface IdentityRecord {
  address: string;
  ens_name: string | null;
  bns_name: string | null;
  world_id_verified: boolean;
}

export interface ErpSyncRecord {
  erp_system: string;
  last_sync: string;
  total_transactions_synced: number;
  status: string;
}

export interface StateProposal {
  proposal_id: string;
  trigger_id: string;
  proposed_state: string;
  timelock_end_block: number;
  status: 'Pending' | 'Approved' | 'Executed';
  tee_attestation: string;
  yield_routing: string;
  capital_status: 'TransitBond' | 'Escrow';
}

export interface SettlementEnvelope {
  protocol: 'ISO20022' | 'PAPSS' | 'BRICS';
  payload: any;
  raw_payload_bytes: string;
  ingress_timestamp: string;
}


/**
 * Returns the correct Gateway API route.
 */
export const getGatewayUrl = (service: string, environment: string): string => {
  const isProduction = environment === 'production';
  const baseUrl = isProduction
    ? 'https://gateway.conxian.com'
    : 'http://localhost:8080';

  return `${baseUrl}/api/v1/${service}`;
};

const currentEnv = process.env.NODE_ENV === 'production' ? 'production' : 'local';

// Sovereign Service Base URLs
export const BISQ_API_URL = getGatewayUrl('bisq', currentEnv);
export const RGB_API_URL = getGatewayUrl('rgb', currentEnv);
export const BITVM_API_URL = getGatewayUrl('bitvm', currentEnv);
export const BITVM2_API_URL = getGatewayUrl('bitvm2', currentEnv);
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
export const CORE_DAO_API_URL = getGatewayUrl('core-dao', currentEnv);
export const LAYERS_API_URL = getGatewayUrl('layers', currentEnv);

// System & Monitoring Endpoints
export const HEALTH_API_URL = getGatewayUrl('health', currentEnv);
export const STATUS_API_URL = getGatewayUrl('status', currentEnv);
export const COMPLIANCE_API_URL = getGatewayUrl('compliance', currentEnv);
export const METRICS_API_URL = getGatewayUrl('metrics', currentEnv);
export const RESERVES_API_URL = getGatewayUrl('reserves', currentEnv);
export const PRICES_API_URL = getGatewayUrl('prices', currentEnv);
export const AFFILIATES_API_URL = getGatewayUrl('affiliates', currentEnv);
export const MARKETING_API_URL = getGatewayUrl('marketing', currentEnv);
export const RISK_ASSESSMENT_API_URL = getGatewayUrl('risk-assessment', currentEnv);

/**
 * Protocol-Specific Helpers
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

export const getLiquidPeg = async () => {
  const url = `${getGatewayUrl("liquid", currentEnv)}/peg`;
  const response = await fetch(url);
  return response.json();
};

export const getRootstockPowpeg = async () => {
  const url = `${getGatewayUrl("rootstock", currentEnv)}/powpeg`;
  const response = await fetch(url);
  return response.json();
};

export const getBabylonStaking = async () => {
  const url = `${getGatewayUrl("babylon", currentEnv)}/staking`;
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

export const checkCompliance = async (address: string) => {
  const url = `${getGatewayUrl("compliance", currentEnv)}/check`;
  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ address }),
  });
  return response.json();
};

export const getCoreDaoStats = async () => {
  const url = `${getGatewayUrl("core-dao", currentEnv)}/stats`;
  const response = await fetch(url);
  return response.json();
};

export const getLorenzoStaking = async () => {
  const url = `${getGatewayUrl("lorenzo", currentEnv)}/stats`;
  const response = await fetch(url);
  return response.json();
};

export const getHemiStatus = async () => {
  const url = `${getGatewayUrl("hemi", currentEnv)}/status`;
  const response = await fetch(url);
  return response.json();
};

export const getB2Status = async () => {
  const url = `${getGatewayUrl("b2network", currentEnv)}/status`;
  const response = await fetch(url);
  return response.json();
};

export const getCitreaProof = async (batchId: string) => {
  const url = `${getGatewayUrl("citrea", currentEnv)}/proof/${batchId}`;
  const response = await fetch(url);
  return response.json();
};

export const getBobInfo = async () => {
  const url = `${getGatewayUrl("bob", currentEnv)}/info`;
  const response = await fetch(url);
  return response.json();
};

export const getMerlinStats = async () => {
  const url = `${getGatewayUrl("merlin", currentEnv)}/stats`;
  const response = await fetch(url);
  return response.json();
};

export const getMezoYield = async () => {
  const url = `${getGatewayUrl("mezo", currentEnv)}/yield`;
  const response = await fetch(url);
  return response.json();
};

export const getNubitDaInfo = async () => {
  const url = `${getGatewayUrl("nubit", currentEnv)}/da`;
  const response = await fetch(url);
  return response.json();
};

export const getBisonStats = async () => {
  const url = `${getGatewayUrl("bison", currentEnv)}/stats`;
  const response = await fetch(url);
  return response.json();
};

export const getZuluInfo = async () => {
  const url = `${getGatewayUrl("zulu", currentEnv)}/info`;
  const response = await fetch(url);
  return response.json();
};

export const getBotanixStats = async () => {
  const url = `${getGatewayUrl("botanix", currentEnv)}/stats`;
  const response = await fetch(url);
  return response.json();
};

export const getBitlayerInfo = async () => {
  const url = `${getGatewayUrl("bitlayer", currentEnv)}/info`;
  const response = await fetch(url);
  return response.json();
};

export const getAlpenStats = async () => {
  const url = `${getGatewayUrl("alpen", currentEnv)}/stats`;
  const response = await fetch(url);
  return response.json();
};

export const getTaprootAssetsStats = async () => {
  const url = `${getGatewayUrl("taproot-assets", currentEnv)}/stats`;
  const response = await fetch(url);
  return response.json();
};

export const getBitvm2Info = async () => {
  const url = `${getGatewayUrl("bitvm2", currentEnv)}/info`;
  const response = await fetch(url);
  return response.json();
};

export interface Bitvm2StateRootVerificationResult {
  state_root: string;
  verified: boolean;
  proof_system?: string;
  curve?: string;
  error?: string;
}

export const verifyBitvm2StateRoot = async (
  stateRoot: string,
  proof: string,
  publicInputs?: string[],
): Promise<Bitvm2StateRootVerificationResult> => {
  const url = `${getGatewayUrl("bitvm2", currentEnv)}/verify-state-root`;
  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      state_root: stateRoot,
      proof,
      public_inputs: publicInputs,
    }),
  });
  return response.json() as Promise<Bitvm2StateRootVerificationResult>;
};

export const getRiskAssessment = async () => {
  const url = getGatewayUrl("risk-assessment", currentEnv);
  const response = await fetch(url);
  return response.json();
};

export const resolveIdentity = async (query: string): Promise<IdentityRecord> => {
  const url = `${getGatewayUrl("identity", currentEnv)}/${query}`;
  const response = await fetch(url);
  return response.json() as Promise<IdentityRecord>;
};

export const syncErpData = async (system: string): Promise<ErpSyncRecord> => {
  const url = `${getGatewayUrl("erp", currentEnv)}/sync`;
  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ system }),
  });
  return response.json() as Promise<ErpSyncRecord>;
};

export const getFinancials = async (): Promise<FinancialMetrics> => {
  const url = getGatewayUrl("financials", currentEnv);
  const response = await fetch(url);
  return response.json() as Promise<FinancialMetrics>;
};

export const getCjcsSpec = async () => {
  const url = getGatewayUrl("spec/cjcs", currentEnv);
  const response = await fetch(url);
  return response.json();
};

export const getDlcBondInfo = async (bondId: string) => {
  const url = `${getGatewayUrl("finance/bond", currentEnv)}/${bondId}`;
  const response = await fetch(url);
  return response.json();
};

export const commitState = async (stateRoot: string) => {
  const url = `${getGatewayUrl("state", currentEnv)}/commit`;
  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ state_root: stateRoot }),
  });
  return response.json();
};

export const verifyZkmlProof = async (proof: string) => {
  const url = `${getGatewayUrl("compliance", currentEnv)}/zkml-verify`;
  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ proof }),
  });
  return response.json();
};

/**
 * Monitoring & Aggregated Stats Helpers
 */
export const getServiceStatus = async (service: string): Promise<ServiceStatus> => {
  const url = getGatewayUrl(service, currentEnv);
  const response = await fetch(url);
  return response.json();
};

export const getLayers = async (): Promise<Record<string, ServiceStatus>> => {
  const url = getGatewayUrl("layers", currentEnv);
  const response = await fetch(url);
  return response.json();
};

export const getReserves = async () => {
  const url = getGatewayUrl("reserves", currentEnv);
  const response = await fetch(url);
  return response.json();
};

export const getSystemStatus = async () => {
  const url = getGatewayUrl("status", currentEnv);
  const response = await fetch(url);
  return response.json();
};

export const getHealth = async () => {
  const url = getGatewayUrl("health", currentEnv);
  const response = await fetch(url);
  return response.json();
};

export const getSettlementProposals = async (): Promise<StateProposal[]> => {
  const url = `${getGatewayUrl("settlement", currentEnv)}/proposals`;
  const response = await fetch(url);
  return response.json();
};

export const submitIso20022Settlement = async (payload: any): Promise<StateProposal> => {
  const url = `${getGatewayUrl("settlement", currentEnv)}/iso20022`;
  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  return response.json();
};

export const submitPapssSettlement = async (payload: any): Promise<StateProposal> => {
  const url = `${getGatewayUrl("settlement", currentEnv)}/papss`;
  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  return response.json();
};

export const submitBricsSettlement = async (payload: any): Promise<StateProposal> => {
  const url = `${getGatewayUrl("settlement", currentEnv)}/brics`;
  const response = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(payload),
  });
  return response.json();
};

export interface SabWallet {
  address: string;
  role: string;
  owner: string;
  status: string;
  quorum?: string;
  spending_limit_usd?: number;
}

export const getSabWallets = async (): Promise<SabWallet[]> => {
  const url = getGatewayUrl("sab/wallets", currentEnv);
  const response = await fetch(url);
  return response.json() as Promise<SabWallet[]>;
};
