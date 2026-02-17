/**
 * Conxian Gateway Network Routing
 * Replaces legacy Anya-core and OPSource endpoints.
 */

export const getGatewayUrl = (service: string, environment: string = 'development'): string => {
  // Support seamless switching between local and production execution
  const isProduction = environment === 'production' || process.env.NODE_ENV === 'production';
  const baseUrl = isProduction
    ? 'https://gateway.conxian.com'
    : 'http://localhost:8080';

  // All sovereign services (Bisq, RGB, BitVM, Changelly) now route through the unified gateway
  return `${baseUrl}/api/v1/${service}`;
};

// Sovereign Service Base URLs
export const BISQ_API_URL = getGatewayUrl('bisq');
export const RGB_API_URL = getGatewayUrl('rgb');
export const BITVM_API_URL = getGatewayUrl('bitvm');
export const CHANGELLY_API_URL = getGatewayUrl('changelly');

/**
 * Legacy routing (Deprecated)
 * Anya-core: http://anya-core:8080
 * OPSource: http://opsource:3000
 */
