import client from './client'

export interface DataSource {
  id: number
  name: string
  source_type: string
  url: string
  interval_seconds: number
  config: Record<string, unknown> | null
  enabled: boolean
  last_fetched_at: string | null
  article_count: number | null
  created_at: string
  updated_at: string
}

export interface CreateSourceRequest {
  name: string
  source_type: string
  url: string
  interval_seconds?: number
  config?: Record<string, unknown>
}

export interface UpdateSourceRequest {
  name?: string
  source_type?: string
  url?: string
  interval_seconds?: number
  config?: Record<string, unknown>
  enabled?: boolean
}

export const sourceApi = {
  list: () => client.get<{ data: DataSource[] }>('/sources').then((r) => r.data.data),

  get: (id: number) => client.get<{ data: DataSource }>(`/sources/${id}`).then((r) => r.data.data),

  create: (data: CreateSourceRequest) =>
    client.post<{ data: DataSource }>('/sources', data).then((r) => r.data.data),

  update: (id: number, data: UpdateSourceRequest) =>
    client.post<{ data: DataSource }>(`/sources/update/${id}`, data).then((r) => r.data.data),

  delete: (id: number) => client.post(`/sources/delete/${id}`).then((r) => r.data),

  fetch: (id: number) => client.post(`/sources/fetch/${id}`).then((r) => r.data)
}
