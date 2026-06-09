import { useState, useEffect, useCallback } from 'react'
import { keywordApi, type Keyword } from '../api/keywords'
import { useToast } from '../components/Toast'
import Empty from '../components/Empty'

interface FormState {
  keyword: string
  case_sensitive: boolean
  std_multiplier: number
  min_hot_count: number
}

const EMPTY_FORM: FormState = {
  keyword: '',
  case_sensitive: false,
  std_multiplier: 2.0,
  min_hot_count: 3
}

export default function Keywords() {
  const [keywords, setKeywords] = useState<Keyword[]>([])
  const [loading, setLoading] = useState(true)
  const [showModal, setShowModal] = useState(false)
  const [editingId, setEditingId] = useState<number | null>(null)
  const [form, setForm] = useState<FormState>(EMPTY_FORM)
  const [submitting, setSubmitting] = useState(false)
  const toast = useToast()

  const load = useCallback(() => {
    keywordApi
      .list()
      .then(setKeywords)
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

  function openEdit(item: Keyword) {
    setEditingId(item.id)
    setForm({
      keyword: item.keyword,
      case_sensitive: item.case_sensitive,
      std_multiplier: item.std_multiplier,
      min_hot_count: item.min_hot_count
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
    if (!form.keyword.trim()) {
      toast.error('请输入关键词')
      return
    }
    setSubmitting(true)
    try {
      if (editingId !== null) {
        await keywordApi.update(editingId, {
          keyword: form.keyword.trim(),
          case_sensitive: form.case_sensitive,
          std_multiplier: form.std_multiplier,
          min_hot_count: form.min_hot_count
        })
        toast.success('关键词已更新')
      } else {
        await keywordApi.create({
          keyword: form.keyword.trim(),
          case_sensitive: form.case_sensitive,
          std_multiplier: form.std_multiplier,
          min_hot_count: form.min_hot_count
        })
        toast.success('关键词已添加')
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
    if (editingId === null) return
    if (!window.confirm('确定要删除该关键词吗？')) return
    try {
      await keywordApi.delete(editingId)
      toast.success('关键词已删除')
      closeModal()
      load()
    } catch {
      // error handled by interceptor
    }
  }

  async function handleToggle(item: Keyword) {
    const action = item.enabled ? '暂停' : '启用'
    try {
      await keywordApi.update(item.id, { enabled: !item.enabled })
      toast.success(`关键词已${action}`)
      load()
    } catch {
      // error handled by interceptor
    }
  }

  if (loading) {
    return <p style={{ color: 'var(--color-muted)', padding: 16 }}>加载中...</p>
  }

  return (
    <div>
      <div className="panel">
        <div className="panel-header">
          <span className="panel-title">关键词管理</span>
          <button className="btn btn-primary btn-sm" onClick={openAdd}>
            + 添加关键词
          </button>
        </div>
        {keywords.length === 0 ? (
          <div style={{ padding: 24 }}>
            <Empty description="暂无关键词" actionText="+ 添加关键词" onAction={openAdd} />
          </div>
        ) : (
          <div className="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>关键词</th>
                  <th>大小写</th>
                  <th>标准差倍数</th>
                  <th>最小计数</th>
                  <th>24h 命中</th>
                  <th>状态</th>
                  <th>操作</th>
                </tr>
              </thead>
              <tbody>
                {keywords.map((k) => (
                  <tr key={k.id}>
                    <td className="mono" style={{ color: 'var(--color-fg)' }}>
                      {k.keyword}
                    </td>
                    <td>{k.case_sensitive ? '是' : '否'}</td>
                    <td>{k.std_multiplier}</td>
                    <td>{k.min_hot_count}</td>
                    <td className="mono" style={{ color: 'var(--color-success)' }}>
                      {k.hit_count_24h ?? '—'}
                    </td>
                    <td>
                      <span className={k.enabled ? 'badge badge-success' : 'badge badge-neutral'}>
                        {k.enabled ? '启用' : '暂停'}
                      </span>
                    </td>
                    <td>
                      <div style={{ display: 'flex', gap: 8 }}>
                        <button
                          className="btn btn-ghost btn-sm"
                          onClick={() => handleToggle(k)}
                        >
                          {k.enabled ? '暂停' : '启用'}
                        </button>
                        <button
                          className="btn btn-ghost btn-sm"
                          onClick={() => openEdit(k)}
                        >
                          编辑
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
          <h2>{editingId !== null ? '编辑关键词' : '添加关键词'}</h2>
          <div className="field">
            <label>关键词</label>
            <input
              value={form.keyword}
              onChange={(e) => setField('keyword', e.target.value)}
              placeholder="例如：AI 热点"
            />
          </div>
          <div className="field">
            <label>大小写敏感</label>
            <select
              value={form.case_sensitive ? '1' : '0'}
              onChange={(e) => setField('case_sensitive', e.target.value === '1')}
            >
              <option value="0">否</option>
              <option value="1">是</option>
            </select>
          </div>
          <div className="field">
            <label>标准差倍数</label>
            <input
              type="number"
              step="0.1"
              min="0"
              value={form.std_multiplier}
              onChange={(e) => setField('std_multiplier', parseFloat(e.target.value) || 0)}
            />
            <div className="field-help">当前计数超过均值+N倍标准差时触发热点</div>
          </div>
          <div className="field">
            <label>最小触发计数</label>
            <input
              type="number"
              step="1"
              min="1"
              value={form.min_hot_count}
              onChange={(e) => setField('min_hot_count', parseInt(e.target.value) || 0)}
            />
            <div className="field-help">计数低于此值不计为热点</div>
          </div>
          <div className="modal-actions">
            {editingId !== null && (
              <button
                className="btn btn-danger btn-sm"
                onClick={handleDelete}
                style={{ marginRight: 'auto' }}
              >
                删除
              </button>
            )}
            <button className="btn btn-ghost btn-sm" onClick={closeModal}>
              取消
            </button>
            <button
              className="btn btn-primary btn-sm"
              disabled={submitting || !form.keyword.trim()}
              onClick={handleSubmit}
            >
              {submitting ? '提交中...' : editingId !== null ? '确认修改' : '确认添加'}
            </button>
          </div>
        </div>
      </div>
    </div>
  )
}
