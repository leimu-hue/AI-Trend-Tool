import client from './client'

export interface TokenInfo {
  id: number
  name: string
  token?: string
  last_used_at: string | null
  expires_at: string | null
  revoked: boolean
  created_at: string
}

export interface CreateTokenRequest {
  name: string
  expires_at?: string | null
}

export interface CreateTokenResponse {
  id: number
  name: string
  token: string
  expires_at: string | null
}

export const tokenApi = {
  list: () => client.get<TokenInfo[]>('/tokens').then((r) => r.data),

  create: (data: CreateTokenRequest) =>
    client.post<CreateTokenResponse>('/tokens', data).then((r) => r.data),

  revoke: (id: number) => client.post(`/tokens/revoke/${id}`).then((r) => r.data)
}
