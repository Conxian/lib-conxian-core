import sys

content = open('services/network.ts').read()

partner_intake_ts = """
/**
 * Partner Intake Endpoints
 */
export interface PartnerLead {
  id: string;
  partner_name: string;
  contact_name: string;
  contact_email: string;
  company_name?: string;
  notes?: string;
  status: 'new' | 'assigned' | 'in_progress' | 'escalated' | 'closed';
  owner?: string;
}

export const createPartnerLead = async (lead: Partial<PartnerLead>, idempotencyKey: string, apiKey: string) => {
  const url = `${getGatewayUrl("intake/partner", currentEnv)}`;
  const response = await fetch(url, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      "X-Partner-Intake-Key": apiKey,
      "Idempotency-Key": idempotencyKey
    },
    body: JSON.stringify(lead),
  });
  return response.json();
};

export const getPartnerLead = async (id: string, apiKey: string) => {
  const url = `${getGatewayUrl("intake/partner", currentEnv)}/${id}`;
  const response = await fetch(url, {
    headers: { "X-Partner-Intake-Key": apiKey }
  });
  return response.json();
};

export const listPartnerLeads = async (apiKey: string, filters?: { status?: string; owner?: string }) => {
  let url = `${getGatewayUrl("intake/partner", currentEnv)}`;
  const params = new URLSearchParams();
  if (filters?.status) params.append("status", filters.status);
  if (filters?.owner) params.append("owner", filters.owner);
  if (params.toString()) url += `?${params.toString()}`;

  const response = await fetch(url, {
    headers: { "X-Partner-Intake-Key": apiKey }
  });
  return response.json();
};
"""

if 'export const createPartnerLead' not in content:
    content += partner_intake_ts

with open('services/network.ts', 'w') as f:
    f.write(content)
