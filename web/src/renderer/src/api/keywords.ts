import client from './client'

export interface Keyword {
  id: number
  keyword: string
  case_sensitive: boolean
  std_multiplier: number
  min_hot_count: number
  enabled: boolean
  hit_count_24h: number | null
  created_at: string
  updated_at: string
}

export interface CreateKeywordRequest {
  keyword: string
  case_sensitive?: boolean
  std_multiplier?: number
  min_hot_count?: number
}

export interface UpdateKeywordRequest {
  keyword?: string
  case_sensitive?: boolean
  std_multiplier?: number
  min_hot_count?: number
  enabled?: boolean
}

export const keywordApi = {
  list: () => client.get<{ data: Keyword[] }>('/keywords').then((r) => r.data.data),

  get: (id: number) => client.get<{ data: Keyword }>(`/keywords/${id}`).then((r) => r.data.data),

  create: (data: CreateKeywordRequest) =>
    client.post<{ data: Keyword }>('/keywords', data).then((r) => r.data.data),

  update: (id: number, data: UpdateKeywordRequest) =>
    client.post<{ data: Keyword }>(`/keywords/update/${id}`, data).then((r) => r.data.data),

  delete: (id: number) => client.post(`/keywords/delete/${id}`).then((r) => r.data)
}
