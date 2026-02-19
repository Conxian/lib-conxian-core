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

// System & Monitoring Endpoints
export const HEALTH_API_URL = getGatewayUrl('health', currentEnv);
export const STATUS_API_URL = getGatewayUrl('status', currentEnv);
export const COMPLIANCE_API_URL = getGatewayUrl('compliance', currentEnv);
export const METRICS_API_URL = getGatewayUrl('metrics', currentEnv);
