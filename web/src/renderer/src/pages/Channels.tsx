import { useState, useEffect, useCallback } from 'react'
import { channelApi, type PushChannel } from '../api/channels'
import { useToast } from '../components/Toast'
import Empty from '../components/Empty'
import Confirm from '../components/Confirm'

interface FormState {
  name: string
  channel_type: string
  webhook_url: string
}

const EMPTY_FORM: FormState = {
  name: '',
  channel_type: 'webhook',
  webhook_url: ''
}

function maskUrl(url: string): string {
  try {
    const u = new URL(url)
    return `${u.protocol}//${u.host}/****`
  } catch {
    if (url.length > 40) return url.slice(0, 40) + '...'
    return url
  }
}

function extractUrl(config: string | Record<string, unknown> | null): string {
  if (!config) return ''
  let obj: Record<string, unknown>
  if (typeof config === 'string') {
    try { obj = JSON.parse(config) } catch { return '' }
  } else {
    obj = config
  }
  if (typeof obj.url === 'string') return obj.url
  return ''
}

export default function Channels() {
  const [channels, setChannels] = useState<PushChannel[]>([])
  const [loading, setLoading] = useState(true)
  const [showModal, setShowModal] = useState(false)
  const [editingId, setEditingId] = useState<number | null>(null)
  const [form, setForm] = useState<FormState>(EMPTY_FORM)
  const [submitting, setSubmitting] = useState(false)
  const [deleteTarget, setDeleteTarget] = useState<PushChannel | null>(null)
  const [deleting, setDeleting] = useState(false)
  const toast = useToast()

  const load = useCallback(() => {
    channelApi
      .list()
      .then(setChannels)
      .catch(() => {})
      .finally(() => setLoading(false))
  }, [])

  useEffect(() => {
    load()
  }, [load])

  function openAdd() {
    setEditingId(null)
    setForm(EMPTY_FORM)
    setShowModal(true)
  }

  function openEdit(item: PushChannel) {
    setEditingId(item.id)
    setForm({
      name: item.name,
      channel_type: item.channel_type,
      webhook_url: extractUrl(item.config)
    })
    setShowModal(true)
  }

  function closeModal() {
    setShowModal(false)
    setEditingId(null)
  }

  function handleOverlayClick(e: React.MouseEvent) {
    if (e.target === e.currentTarget) closeModal()
  }

  function setField<K extends keyof FormState>(key: K, value: FormState[K]) {
    setForm((prev) => ({ ...prev, [key]: value }))
  }

  async function handleSubmit() {
    if (!form.name.trim()) {
      toast.error('请输入渠道名称')
      return
    }
    if (!form.webhook_url.trim()) {
      toast.error('请输入 Webhook URL')
      return
    }
    setSubmitting(true)
    const config = JSON.stringify({ url: form.webhook_url.trim() })
    try {
      if (editingId !== null) {
        await channelApi.update(editingId, {
          name: form.name.trim(),
          channel_type: form.channel_type,
          config
        })
        toast.success('推送渠道已更新')
      } else {
        await channelApi.create({
          name: form.name.trim(),
          channel_type: form.channel_type,
          config
        })
        toast.success('推送渠道已添加')
      }
      closeModal()
      load()
    } catch {
      // error handled by interceptor
    } finally {
      setSubmitting(false)
    }
  }

  async function handleDelete() {
    if (!deleteTarget) return
    setDeleting(true)
    try {
      await channelApi.delete(deleteTarget.id)
      toast.success('推送渠道已删除')
      setDeleteTarget(null)
      load()
    } catch {
      // error handled by interceptor
    } finally {
      setDeleting(false)
    }
  }

  async function handleTest(item: PushChannel) {
    try {
      await channelApi.test(item.id)
      toast.success('测试消息已发送')
    } catch {
      toast.error('测试功能暂不可用')
    }
  }

  function formatLastPushed(d: string | null): string {
    if (!d) return '—'
    try {
      return new Date(d).toLocaleString('zh-CN', { hour12: false })
    } catch {
      return d
    }
  }

  if (loading) {
    return <p style={{ color: 'var(--color-muted)', padding: 16 }}>加载中...</p>
  }

  return (
    <div>
      <div className="panel">
        <div className="panel-header">
          <span className="panel-title">推送渠道管理</span>
          <button className="btn btn-primary btn-sm" onClick={openAdd}>
            + 添加渠道
          </button>
        </div>
        {channels.length === 0 ? (
          <div style={{ padding: 24 }}>
            <Empty description="暂无推送渠道" actionText="+ 添加渠道" onAction={openAdd} />
          </div>
        ) : (
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>名称</th>
                  <th>类型</th>
                  <th>Webhook URL</th>
                  <th>推送次数</th>
                  <th>上次推送</th>
                  <th>状态</th>
                  <th>操作</th>
                </tr>
              </thead>
              <tbody>
                {channels.map((c) => (
                  <tr key={c.id}>
                    <td style={{ color: 'var(--color-fg)' }}>{c.name}</td>
                    <td>
                      <span className="badge badge-neutral">{c.channel_type}</span>
                    </td>
                    <td className="mono truncate">{maskUrl(extractUrl(c.config))}</td>
                    <td>{c.push_count ?? '—'}</td>
                    <td className="mono">{formatLastPushed(c.last_pushed_at)}</td>
                    <td>
                      <span className={c.enabled ? 'badge badge-success' : 'badge badge-neutral'}>
                        {c.enabled ? '启用' : '暂停'}
                      </span>
                    </td>
                    <td>
                      <div style={{ display: 'flex', gap: 8 }}>
                        <button
                          className="btn btn-ghost btn-sm"
                          onClick={() => handleTest(c)}
                        >
                          测试
                        </button>
                        <button
                          className="btn btn-ghost btn-sm"
                          onClick={() => openEdit(c)}
                        >
                          编辑
                        </button>
                        <button
                          className="btn btn-ghost btn-sm btn-danger"
                          onClick={() => setDeleteTarget(c)}
                        >
                          删除
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>

      {/* Add/Edit Modal */}
      <div className={`modal-overlay${showModal ? ' open' : ''}`} onClick={handleOverlayClick}>
        <div className="modal">
          <h2>{editingId !== null ? '编辑推送渠道' : '添加推送渠道'}</h2>
          <div className="field">
            <label>渠道名称</label>
            <input
              value={form.name}
              onChange={(e) => setField('name', e.target.value)}
              placeholder="例如：钉钉告警群"
            />
          </div>
          <div className="field">
            <label>类型</label>
            <select
              value={form.channel_type}
              onChange={(e) => setField('channel_type', e.target.value)}
            >
              <option value="webhook">Webhook</option>
            </select>
          </div>
          <div className="field">
            <label>Webhook URL</label>
            <input
              value={form.webhook_url}
              onChange={(e) => setField('webhook_url', e.target.value)}
              placeholder="https://hooks.example.com/path"
              type="url"
            />
          </div>
          <div className="modal-actions">
            <button className="btn btn-ghost btn-sm" onClick={closeModal}>
              取消
            </button>
            <button
              className="btn btn-primary btn-sm"
              disabled={submitting || !form.name.trim() || !form.webhook_url.trim()}
              onClick={handleSubmit}
            >
              {submitting ? '提交中...' : editingId !== null ? '确认修改' : '确认添加'}
            </button>
          </div>
        </div>
      </div>

      <Confirm
        open={deleteTarget !== null}
        title="删除推送渠道"
        message={`确定要删除推送渠道「${deleteTarget?.name}」吗？此操作不可撤销。`}
        confirmText="删除"
        danger
        onConfirm={handleDelete}
        onCancel={() => setDeleteTarget(null)}
      />
    </div>
  )
}
