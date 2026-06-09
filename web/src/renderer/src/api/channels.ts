import client from './client'

export interface PushChannel {
  id: number
  name: string
  channel_type: string
  config: Record<string, unknown>
  enabled: boolean
  push_count: number | null
  last_pushed_at: string | null
  created_at: string
  updated_at: string
}

export interface CreateChannelRequest {
  name: string
  channel_type: string
  config: Record<string, unknown>
}

export interface UpdateChannelRequest {
  name?: string
  channel_type?: string
  config?: Record<string, unknown>
  enabled?: boolean
}

export const channelApi = {
  list: () => client.get<{ data: PushChannel[] }>('/channels').then((r) => r.data.data),

  get: (id: number) => client.get<{ data: PushChannel }>(`/channels/${id}`).then((r) => r.data.data),

  create: (data: CreateChannelRequest) =>
    client.post<{ data: PushChannel }>('/channels', data).then((r) => r.data.data),

  update: (id: number, data: UpdateChannelRequest) =>
    client.post<{ data: PushChannel }>(`/channels/update/${id}`, data).then((r) => r.data.data),

  delete: (id: number) => client.post(`/channels/delete/${id}`).then((r) => r.data),

  test: (id: number) => client.post(`/channels/test/${id}`).then((r) => r.data)
}
