import { useState, useEffect, useCallback } from 'react'
import { sourceApi, type DataSource } from '../api/sources'
import { useToast } from '../components/Toast'
import Empty from '../components/Empty'
import Confirm from '../components/Confirm'

interface FormState {
  name: string
  type: string
  url: string
  interval_seconds: string
}

const EMPTY_FORM: FormState = {
  name: '',
  type: 'RSS',
  url: '',
  interval_seconds: '300'
}

export default function Sources() {
  const [sources, setSources] = useState<DataSource[]>([])
  const [loading, setLoading] = useState(true)
  const [showModal, setShowModal] = useState(false)
  const [editingId, setEditingId] = useState<number | null>(null)
  const [form, setForm] = useState<FormState>(EMPTY_FORM)
  const [submitting, setSubmitting] = useState(false)
  const [deleteTarget, setDeleteTarget] = useState<DataSource | null>(null)
  const [deleting, setDeleting] = useState(false)
  const toast = useToast()

  const load = useCallback(() => {
    sourceApi
      .list()
      .then(setSources)
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

  function openEdit(item: DataSource) {
    setEditingId(item.id)
    setForm({
      name: item.name,
      type: item.type,
      url: item.url,
      interval_seconds: String(item.interval_seconds)
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
      toast.error('请输入数据源名称')
      return
    }
    if (!form.url.trim()) {
      toast.error('请输入数据源 URL')
      return
    }
    const intervalSeconds = form.interval_seconds === '' ? 300 : parseInt(form.interval_seconds, 10)
    if (isNaN(intervalSeconds) || intervalSeconds < 30) {
      toast.error('拉取间隔最小为 30 秒')
      return
    }
    setSubmitting(true)
    try {
      if (editingId !== null) {
        await sourceApi.update(editingId, {
          name: form.name.trim(),
          type: form.type,
          url: form.url.trim(),
          interval_seconds: intervalSeconds
        })
        toast.success('数据源已更新')
      } else {
        await sourceApi.create({
          name: form.name.trim(),
          type: form.type,
          url: form.url.trim(),
          interval_seconds: intervalSeconds
        })
        toast.success('数据源已添加')
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
      await sourceApi.delete(deleteTarget.id)
      toast.success('数据源已删除')
      setDeleteTarget(null)
      load()
    } catch {
      // error handled by interceptor
    } finally {
      setDeleting(false)
    }
  }

  async function handleFetch(item: DataSource) {
    try {
      await sourceApi.fetch(item.id)
      toast.success('手动抓取已触发')
    } catch {
      // error handled by interceptor
    }
  }

  function formatLastFetched(d: string | null): string {
    if (!d) return '—'
    try {
      return new Date(d).toLocaleString('zh-CN', { hour12: false })
    } catch {
      return d
    }
  }

  function truncateUrl(url: string): string {
    if (url.length > 50) return url.slice(0, 50) + '...'
    return url
  }

  if (loading) {
    return <p style={{ color: 'var(--color-muted)', padding: 16 }}>加载中...</p>
  }

  return (
    <div>
      <div className="panel">
        <div className="panel-header">
          <span className="panel-title">数据源管理</span>
          <button className="btn btn-primary btn-sm" onClick={openAdd}>
            + 添加数据源
          </button>
        </div>
        {sources.length === 0 ? (
          <div style={{ padding: 24 }}>
            <Empty description="暂无数据源" actionText="+ 添加数据源" onAction={openAdd} />
          </div>
        ) : (
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>名称</th>
                  <th>类型</th>
                  <th>URL</th>
                  <th>间隔</th>
                  <th>文章数</th>
                  <th>上次抓取</th>
                  <th>状态</th>
                  <th>操作</th>
                </tr>
              </thead>
              <tbody>
                {sources.map((s) => (
                  <tr key={s.id}>
                    <td style={{ color: 'var(--color-fg)' }}>{s.name}</td>
                    <td>
                      <span className="badge badge-neutral">{s.type}</span>
                    </td>
                    <td className="mono truncate" title={s.url}>
                      {truncateUrl(s.url)}
                    </td>
                    <td>{s.interval_seconds}s</td>
                    <td>{s.article_count ?? '—'}</td>
                    <td className="mono">{formatLastFetched(s.last_fetched_at)}</td>
                    <td>
                      <span className={s.enabled ? 'badge badge-success' : 'badge badge-danger'}>
                        {s.enabled ? '启用' : '禁用'}
                      </span>
                    </td>
                    <td>
                      <div style={{ display: 'flex', gap: 8 }}>
                        <button
                          className="btn btn-ghost btn-sm"
                          onClick={() => handleFetch(s)}
                        >
                          抓取
                        </button>
                        <button
                          className="btn btn-ghost btn-sm"
                          onClick={() => openEdit(s)}
                        >
                          编辑
                        </button>
                        <button
                          className="btn btn-ghost btn-sm btn-danger"
                          onClick={() => setDeleteTarget(s)}
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
          <h2>{editingId !== null ? '编辑数据源' : '添加数据源'}</h2>
          <div className="field">
            <label>名称</label>
            <input
              value={form.name}
              onChange={(e) => setField('name', e.target.value)}
              placeholder="例如：知乎 AI 热榜"
            />
          </div>
          <div className="field">
            <label>类型</label>
            <select
              value={form.type}
              onChange={(e) => setField('type', e.target.value)}
            >
              <option value="RSS">RSS</option>
              <option value="API">API</option>
              <option value="Atom">Atom</option>
            </select>
          </div>
          <div className="field">
            <label>URL</label>
            <input
              value={form.url}
              onChange={(e) => setField('url', e.target.value)}
              placeholder="https://example.com/rss"
              type="url"
            />
          </div>
          <div className="field">
            <label>拉取间隔（秒）</label>
            <input
              type="number"
              step="1"
              min="30"
              value={form.interval_seconds}
              onChange={(e) => setField('interval_seconds', e.target.value)}
            />
            <div className="field-help">最小 30 秒，默认 300 秒</div>
          </div>
          <div className="modal-actions">
            <button className="btn btn-ghost btn-sm" onClick={closeModal}>
              取消
            </button>
            <button
              className="btn btn-primary btn-sm"
              disabled={submitting || !form.name.trim() || !form.url.trim()}
              onClick={handleSubmit}
            >
              {submitting ? '提交中...' : editingId !== null ? '确认修改' : '确认添加'}
            </button>
          </div>
        </div>
      </div>

      <Confirm
        open={deleteTarget !== null}
        title="删除数据源"
        message={`确定要删除数据源「${deleteTarget?.name}」吗？此操作不可撤销。`}
        confirmText="删除"
        danger
        onConfirm={handleDelete}
        onCancel={() => setDeleteTarget(null)}
      />
    </div>
  )
}
