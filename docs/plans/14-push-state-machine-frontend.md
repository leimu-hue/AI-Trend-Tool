# Push State Machine — 前端变更方案

> **编号**: 14  
> **依赖**: 13-push-state-machine.md (后端变更)  
> **目标**: 前端适配后端新的双层状态机模型

---

## 1. 变更总览

| 文件 | 变更内容 |
|------|---------|
| `web/src/renderer/src/api/queries.ts` | 更新 TypeScript 接口定义，新增 `status`、`last_error` 字段 |
| `web/src/renderer/src/pages/Articles.tsx` | 状态筛选从 boolean 改为多值枚举；Badge 展示 4 种状态 |
| `web/src/renderer/src/pages/Dashboard.tsx` | 热点表推送状态展示增强；最新文章状态列适配 |
| `web/src/renderer/src/styles/index.css` | 新增 `badge-dead`、`badge-info` 样式 |

---

## 2. API 接口层 (`api/queries.ts`)

### 2.1 Article 接口更新

```typescript
// Before
export interface Article {
  id: number
  source_id: number
  link: string
  title: string
  summary: string
  content: string
  published_at: string | null
  fetched_at: string
  processed_at: string | null       // 二元: null = 待处理
}

// After
export interface Article {
  id: number
  source_id: number
  link: string
  title: string
  summary: string
  content: string
  published_at: string | null
  fetched_at: string
  processed_at: string | null       // 保留向后兼容
  status: string                    // 新增: 'pending' | 'processing' | 'matched' | 'skipped'
}
```

### 2.2 PushRecord 接口更新

```typescript
// Before
export interface PushRecord {
  id: number
  hot_event_id: number
  channel_id: number
  channel_name: string
  status: string                    // 'pending' | 'processing' | 'success' | 'failed'
  retry_count: number
  next_retry_at: string | null
  created_at: string
  updated_at: string
}

// After
export interface PushRecord {
  id: number
  hot_event_id: number
  channel_id: number
  channel_name: string
  status: string                    // 'pending' | 'processing' | 'success' | 'failed' | 'dead'
  retry_count: number
  next_retry_at: string | null
  last_error: string | null         // 新增: 失败原因
  created_at: string
  updated_at: string
}
```

### 2.3 getArticles 参数更新

```typescript
// Before
getArticles: (params?: {
  page?: number; per_page?: number; source_id?: number; processed?: boolean
}) => ...

// After — 同时支持 processed (向后兼容) 和 status (新)
getArticles: (params?: {
  page?: number
  per_page?: number
  source_id?: number
  status?: string        // 新增: 'pending' | 'processing' | 'matched' | 'skipped'
  processed?: boolean    // 保留向后兼容，后续废弃
}) =>
  client.get<{ data: PaginatedResponse<Article> }>('/articles', { params })
    .then((r) => r.data.data),
```

---

## 3. Articles 页面 (`pages/Articles.tsx`)

### 3.1 状态筛选器变更

**当前**: `processedFilter: boolean | undefined`，二选一 (待处理 / 已处理)

**改为**: `statusFilter: string | undefined`，五种选项 (全部 / 待处理 / 处理中 / 已匹配 / 已跳过)

```typescript
// Before
const [processedFilter, setProcessedFilter] = useState<boolean | undefined>(undefined)

// After
const [statusFilter, setStatusFilter] = useState<string | undefined>(undefined)
```

Select 组件选项:

```tsx
<Select
  className="filter-select"
  popupClassName="filter-select-dropdown"
  size="small"
  value={statusFilter ?? ''}
  onChange={(val) => { setStatusFilter(val === '' ? undefined : val) }}
  options={[
    { value: '', label: '全部状态' },
    { value: 'pending', label: '待处理' },
    { value: 'processing', label: '处理中' },
    { value: 'matched', label: '已匹配' },
    { value: 'skipped', label: '已跳过' },
  ]}
/>
```

### 3.2 load 函数参数更新

```typescript
// Before
const load = useCallback(
  async (p: number, srcFilter: number | undefined, procFilter: boolean | undefined) => {
    const result = await queryApi.getArticles({
      page: p, per_page: PER_PAGE, source_id: srcFilter, processed: procFilter,
    })
    // ...
  }, []
)

// After
const load = useCallback(
  async (p: number, srcFilter: number | undefined, stFilter: string | undefined) => {
    const result = await queryApi.getArticles({
      page: p, per_page: PER_PAGE, source_id: srcFilter, status: stFilter,
    })
    // ...
  }, []
)
```

### 3.3 状态 Badge 展示

**当前**: 二元显示 (已处理 / 待处理)

**改为**: 四种状态 Badge，带颜色和提示

```tsx
// 状态映射工具函数
function articleStatusBadge(status: string) {
  switch (status) {
    case 'pending':
      return { cls: 'badge-warn', label: '待处理' }
    case 'processing':
      return { cls: 'badge-info', label: '处理中' }
    case 'matched':
      return { cls: 'badge-success', label: '已匹配' }
    case 'skipped':
      return { cls: 'badge-muted', label: '已跳过' }
    default:
      // 向后兼容: 无 status 字段时用 processed_at
      return { cls: 'badge-warn', label: '未知' }
  }
}

// 在 table cell 中使用
<td>
  {(() => {
    const badge = articleStatusBadge(a.status)
    return (
      <span className={`badge ${badge.cls}`}>
        <span className={`badge-dot dot-show ${
          badge.cls === 'badge-success' ? 'success-dot' :
          badge.cls === 'badge-warn' ? 'warn-dot' :
          badge.cls === 'badge-info' ? 'info-dot' : 'muted-dot'
        }`} />
        {badge.label}
      </span>
    )
  })()}
</td>
```

### 3.4 useEffect 依赖更新

```typescript
// Before
useEffect(() => {
  load(1, sourceFilter, processedFilter)
}, [load, sourceFilter, processedFilter])

// After
useEffect(() => {
  load(1, sourceFilter, statusFilter)
}, [load, sourceFilter, statusFilter])
```

---

## 4. Dashboard 页面 (`pages/Dashboard.tsx`)

### 4.1 推送状态映射增强

**当前**: 只区分 `sent` / `pending`

**改为**: 显示更精细的状态 (`success` / `failed` / `dead` / `pending` / `processing`)

```typescript
// Before
const statusMap: Record<number, string> = {}
pushResults.forEach((pr, idx) => {
  const heId = hs.items[idx].id
  if (pr.status === 'fulfilled') {
    const hasSent = pr.value.some((r) => r.status === 'sent')
    statusMap[heId] = hasSent ? 'sent' : 'pending'
  } else {
    statusMap[heId] = 'unknown'
  }
})

// After — 聚合多个 channel 的推送状态
const statusMap: Record<number, { status: string; errors: string[] }> = {}
pushResults.forEach((pr, idx) => {
  const heId = hs.items[idx].id
  if (pr.status === 'fulfilled') {
    const records = pr.value
    const hasDead = records.some((r) => r.status === 'dead')
    const hasFailed = records.some((r) => r.status === 'failed')
    const allSuccess = records.length > 0 && records.every((r) => r.status === 'success')
    const errors = records
      .filter((r) => r.last_error)
      .map((r) => `${r.channel_name}: ${r.last_error}`)

    let status: string
    if (allSuccess) status = 'success'
    else if (hasDead) status = 'dead'
    else if (hasFailed) status = 'failed'
    else status = 'pending'

    statusMap[heId] = { status, errors }
  } else {
    statusMap[heId] = { status: 'unknown', errors: [] }
  }
})
```

### 4.2 热点表推送状态 Badge

```tsx
function pushStatusBadge(info: { status: string; errors: string[] }) {
  switch (info.status) {
    case 'success':
      return <span className="badge badge-success">已推送</span>
    case 'failed':
      return (
        <span className="badge badge-warn" title={info.errors.join('\n')}>
          推送失败
        </span>
      )
    case 'dead':
      return (
        <span className="badge badge-dead" title={info.errors.join('\n')}>
          已放弃
        </span>
      )
    case 'pending':
      return <span className="badge badge-info">待推送</span>
    default:
      return <span className="badge badge-muted">未知</span>
  }
}

// 在 table cell 中使用
<td>{pushStatusBadge(pushStatusMap[h.id] || { status: 'pending', errors: [] })}</td>
```

### 4.3 最新文章状态列适配

```tsx
// Before
<td>
  <span className={`badge ${a.processed_at ? 'badge-success' : 'badge-warn'}`}>
    {a.processed_at ? '已处理' : '待处理'}
  </span>
</td>

// After — 复用 articleStatusBadge 函数 (从 Articles.tsx 提取为共享工具)
<td>
  {(() => {
    const badge = articleStatusBadge(a.status)
    return <span className={`badge ${badge.cls}`}>{badge.label}</span>
  })()}
</td>
```

---

## 5. CSS 样式 (`styles/index.css`)

### 5.1 新增 Badge 样式

```css
/* 新增: badge-info (处理中 / 待推送) */
.badge-info {
  background: rgba(59, 130, 246, 0.12);
  color: #60a5fa;
}

/* 新增: badge-dead (已放弃 / 重试耗尽) */
.badge-dead {
  background: rgba(239, 68, 68, 0.12);
  color: #f87171;
}

/* 新增: badge-muted (已跳过 / 未知) */
.badge-muted {
  background: rgba(255, 255, 255, 0.05);
  color: var(--meta);
}

/* 新增: dot 样式 */
.info-dot {
  background: #60a5fa;
}
.muted-dot {
  background: var(--meta);
}
```

---

## 6. 向后兼容策略

| 阶段 | 后端 API | 前端行为 |
|------|---------|---------|
| **阶段 1** (本次) | 返回 `status` + `processed_at` 双字段 | 优先使用 `status`，fallback 到 `processed_at` |
| **阶段 2** (后续) | 废弃 `processed` 查询参数 | 移除 `processed` 参数，仅使用 `status` |
| **阶段 3** (远期) | 移除 `processed_at` 字段 | 完全依赖 `status` |

前端兼容 fallback 逻辑:

```typescript
function articleStatusBadge(status?: string, processedAt?: string | null) {
  if (status) {
    // 新接口: 使用 status 字段
    switch (status) { /* ... */ }
  } else {
    // 旧接口 fallback
    return processedAt
      ? { cls: 'badge-success', label: '已处理' }
      : { cls: 'badge-warn', label: '待处理' }
  }
}
```

---

## 7. 实施步骤

| # | 任务 | 文件 |
|---|------|------|
| 1 | 更新 `Article` 接口: 添加 `status` 字段 | `api/queries.ts` |
| 2 | 更新 `PushRecord` 接口: 添加 `last_error` 字段 | `api/queries.ts` |
| 3 | 更新 `getArticles` 参数: 添加 `status` 参数 | `api/queries.ts` |
| 4 | 提取 `articleStatusBadge` 工具函数 | 新建 `utils/statusBadge.ts` 或内联 |
| 5 | 更新 Articles 页: 筛选器 + Badge | `pages/Articles.tsx` |
| 6 | 更新 Dashboard 页: 推送状态聚合 + Badge | `pages/Dashboard.tsx` |
| 7 | 添加 `badge-info`、`badge-dead`、`badge-muted` CSS | `styles/index.css` |
| 8 | `npm run build` 验证编译通过 | — |

---

## 8. 状态 Badge 颜色规范

### Article 状态

| 状态 | Badge Class | 颜色 | 含义 |
|------|-------------|------|------|
| `pending` | `badge-warn` | 黄色 | 等待处理 |
| `processing` | `badge-info` | 蓝色 | 正在处理 |
| `matched` | `badge-success` | 绿色 | 匹配成功 |
| `skipped` | `badge-muted` | 灰色 | 无匹配跳过 |

### Push Record 状态

| 状态 | Badge Class | 颜色 | 含义 |
|------|-------------|------|------|
| `pending` | `badge-info` | 蓝色 | 等待推送 |
| `processing` | `badge-info` | 蓝色 | 正在推送 |
| `success` | `badge-success` | 绿色 | 推送成功 |
| `failed` | `badge-warn` | 黄色 | 推送失败 (可重试) |
| `dead` | `badge-dead` | 红色 | 重试耗尽 (需人工介入) |
