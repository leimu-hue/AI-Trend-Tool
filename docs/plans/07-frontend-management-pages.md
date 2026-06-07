# 步骤 07：前端管理页面（数据源 + 关键词 + 推送渠道 + Token 设置 + 设置）

## 前置依赖

- 步骤 06 已完成（前端脚手架、路由、API 封装、全局样式、设计 Token）
- `docs/Live-Artifact/index.html` 作为视觉参考

## 目标

完成后拥有 5 个管理页面：
- 数据源管理页（CRUD 表格）
- 关键词管理页（CRUD 表格）
- 推送渠道管理页（CRUD 表格）
- Token 设置页（创建/吊销/复制 Token）
- 系统设置页（只读展示配置参数）

所有页面使用自定义 CSS 组件（`.panel`、`.btn`、`.badge`、`.modal` 等），匹配原型的暗色主题风格。
不使用 Ant Design，保持视觉一致性。

---

## 1. 数据源管理页 `src/pages/Sources.tsx`

### 1.1 页面功能

参考原型 `view-sources` 视图：
- 表格展示所有数据源：名称、类型、URL、间隔(秒)、文章数、上次抓取、状态、操作
- 右上角「+ 添加数据源」按钮 → 打开 Modal 表单
- 每行操作：「抓取」+ 「编辑」按钮
- 删除功能可在编辑弹窗中实现或直接添加删除按钮

### 1.2 表格列（匹配原型）

| 列 | 说明 |
|----|------|
| 名称 | `color: var(--fg)` |
| 类型 | badge-neutral（RSS / API） |
| URL | mono 字体，truncate 截断 |
| 间隔(秒) | mono 字体 |
| 文章数 | mono 字体 |
| 上次抓取 | mono 字体 |
| 状态 | badge-success / badge-danger |
| 操作 | 「抓取」+ 「编辑」按钮 |

### 1.3 表单字段（Modal 弹窗，匹配原型）

| 字段 | 组件 | 验证规则 |
|------|------|--------|
| 名称 | input text | 必填 |
| 类型 | select（RSS/API/Atom） | 必填 |
| URL | input text | 必填，URL 格式 |
| 拉取间隔（秒） | input number | 默认 300，最小 30 |

### 1.4 实现要点

```tsx
// 使用原型 CSS 类名构建页面
<div className="panel">
  <div className="panel-header">
    <span className="panel-title">数据源管理</span>
    <button className="btn btn-primary btn-sm" onClick={() => setModalOpen(true)}>
      + 添加数据源
    </button>
  </div>
  <div className="table-wrap">
    <table>
      <thead>
        <tr>
          <th>名称</th><th>类型</th><th>URL</th><th>间隔(秒)</th>
          <th>文章数</th><th>上次抓取</th><th>状态</th><th>操作</th>
        </tr>
      </thead>
      <tbody>
        {sources.map(s => (
          <tr key={s.id}>
            <td style={{ color: 'var(--fg)' }}>{s.name}</td>
            <td><span className="badge badge-neutral">{s.type.toUpperCase()}</span></td>
            <td><span className="mono truncate" style={{ fontSize: 11, maxWidth: 200 }}>{s.url}</span></td>
            <td><span className="mono">{s.interval_seconds}</span></td>
            {/* ... 文章数需从后端 API 或本地状态获取 */}
            <td>
              <button className="btn btn-ghost btn-sm" onClick={() => handleFetch(s.id)}>抓取</button>
              <button className="btn btn-ghost btn-sm" onClick={() => openEdit(s)}>编辑</button>
            </td>
          </tr>
        ))}
      </tbody>
    </table>
  </div>
</div>

// Modal 弹窗（匹配原型样式）
<div className={`modal-overlay ${modalOpen ? 'open' : ''}`} onClick={handleOverlayClick}>
  <div className="modal">
    <h3>{editingId ? '编辑数据源' : '添加数据源'}</h3>
    <div className="field">
      <label>名称</label>
      <input type="text" placeholder="例如：Hacker News AI" value={form.name} onChange={...} />
    </div>
    <div className="field">
      <label>类型</label>
      <select value={form.type} onChange={...}>
        <option value="rss">RSS</option><option value="api">API</option><option value="atom">Atom</option>
      </select>
    </div>
    <div className="field">
      <label>URL</label>
      <input type="text" placeholder="https://..." value={form.url} onChange={...} />
    </div>
    <div className="field">
      <label>拉取间隔（秒）</label>
      <input type="number" value={form.interval_seconds} onChange={...} />
      <span className="field-help">默认 300 秒（5 分钟）</span>
    </div>
    <div className="modal-actions">
      <button className="btn btn-ghost" onClick={() => setModalOpen(false)}>取消</button>
      <button className="btn btn-primary" onClick={handleSubmit}>确认添加</button>
    </div>
  </div>
</div>
```

> 使用 `useToast()` hook 显示操作结果通知（如「数据源已添加」「手动抓取已触发」）。

---

## 2. 关键词管理页 `src/pages/Keywords.tsx`

### 2.1 表格列（匹配原型 `view-keywords`）

| 列 | 说明 |
|----|------|
| 关键词 | mono 字体，`color: var(--fg)` |
| 大小写 | 「是」/「否」 |
| 标准差倍数 | mono 字体 |
| 最小计数 | mono 字体 |
| 24h 命中 | mono 字体，绿色 |
| 状态 | badge-success（启用）/ badge-neutral（暂停） |
| 操作 | 「暂停/启用」+ 「编辑」按钮 |

### 2.2 表单字段（Modal 弹窗，匹配原型）

| 字段 | 组件 | 验证规则 |
|------|------|--------|
| 关键词 | input text | 必填 |
| 大小写敏感 | select（是/否） | 默认「否」 |
| 标准差倍数 | input number | 默认 2.0，步长 0.1 |
| 最小触发计数 | input number | 默认 3 |

### 2.3 实现要点

与 Sources 页面相同的自定义 CSS 模式。状态切换按钮直接调用 PUT API，无需打开 Modal。使用 `useToast()` 显示操作反馈。

---

## 3. 推送渠道管理页 `src/pages/Channels.tsx`

### 3.1 表格列（匹配原型 `view-channels`）

| 列 | 说明 |
|----|------|
| 名称 | `color: var(--fg)` |
| 类型 | badge-neutral（webhook） |
| Webhook URL | mono 字体，truncate 截断，脱敏显示 |
| 推送次数 | mono 字体 |
| 上次推送 | mono 字体 |
| 状态 | badge-success / badge-neutral |
| 操作 | 「测试」+ 「编辑」按钮 |

### 3.2 表单字段（Modal 弹窗，匹配原型）

| 字段 | 组件 | 验证规则 |
|------|------|--------|
| 名称 | input text | 必填 |
| 类型 | select（webhook） | 必填 |
| Webhook URL | input text | 必填，URL 格式 |

### 3.3 实现要点

- config 字段是 JSON 字符串，表单中拆分为 Webhook URL 单独输入
- 提交时组合回 `config: JSON.stringify({ url: values.webhook_url })`
- 「测试」按钮调用 `POST /channels/{id}/test`（若后端支持）或显示 Toast 提示
- 使用自定义 CSS 组件，与 Sources 页面相同模式

---

## 4. Token 设置页 `src/pages/Tokens.tsx`

### 4.1 页面功能（匹配原型 `view-tokens`）

- 表格展示：名称、最后使用、过期时间、状态、操作
- 有效 Token：「复制」+ 「吊销」按钮
- 已吊销 Token：操作列显示「—」
- 「+ 生成令牌」按钮 → 打开 Modal

### 4.2 表格列（匹配原型）

| 列 | 说明 |
|----|------|
| 名称 | `color: var(--fg)` |
| 最后使用 | mono 字体 |
| 过期时间 | mono 字体，「永久」或日期 |
| 状态 | badge-success（有效）/ badge-danger（已吊销） |
| 操作 | 「复制」+ 「吊销」（有效时） / 「—」（已吊销） |

### 4.3 表单字段（Modal 弹窗，匹配原型）

| 字段 | 组件 | 验证规则 |
|------|------|--------|
| 令牌名称/用途 | input text | 必填 |
| 过期时间 | input date | 可选，留空为永久 |

### 4.4 实现要点

- 创建成功后展示新 Token 明文（仅一次），提供复制按钮
- 吊销前弹出确认：`if (confirm('确定要吊销令牌 "xxx" 吗？'))`
- 使用 `useToast()` 显示「令牌已复制」「已吊销」等反馈
- 使用 `data-token-revoked` 属性控制行内按钮可见性（与原型一致）

---

## 5. 系统设置页 `src/pages/Settings.tsx`

### 5.1 页面功能（匹配原型 `view-settings`）

只读展示后端配置参数，分 4 组卡片：
- **解析器配置**：最大并发抓取数、默认超时时间、默认抓取间隔
- **过滤器配置**：批处理大小、运行间隔、历史窗口、最小历史数据
- **推送器配置**：轮询间隔、最大重试次数、重试基础间隔
- **服务器配置**：监听地址、端口

### 5.2 实现

使用 `.settings-grid` 布局（2 列网格，响应式变为 1 列）：

```tsx
import React, { useEffect, useState } from 'react';
import client from '../api/client';
import { useToast } from '../components/Toast';

interface Settings {
  maxConcurrentFetches: string;
  defaultTimeout: string;
  defaultInterval: string;
  batchSize: string;
  filterIntervalSec: string;
  historyHours: string;
  minHistoryHours: string;
  pusherIntervalSec: string;
  maxRetries: string;
  retryBaseSec: string;
  host: string;
  port: string;
}

const SettingsPage: React.FC = () => {
  const [settings, setSettings] = useState<Settings | null>(null);

  useEffect(() => {
    // 配置 API 可从后端获取，或使用默认值
    client.get('/settings').then(res => setSettings(res.data)).catch(() => {
      // 若后端未实现 /settings API，使用默认值
      setSettings({
        maxConcurrentFetches: '10', defaultTimeout: '30', defaultInterval: '300',
        batchSize: '1000', filterIntervalSec: '300', historyHours: '24',
        minHistoryHours: '6', pusherIntervalSec: '10', maxRetries: '3',
        retryBaseSec: '60', host: '0.0.0.0', port: '8080',
      });
    });
  }, []);

  if (!settings) return <div style={{ color: 'var(--muted)' }}>加载中...</div>;

  return (
    <div className="settings-grid">
      <SettingsGroup title="解析器配置" rows={[
        { label: '最大并发抓取数', value: settings.maxConcurrentFetches },
        { label: '默认超时时间（秒）', value: settings.defaultTimeout },
        { label: '默认抓取间隔（秒）', value: settings.defaultInterval },
      ]} />
      <SettingsGroup title="过滤器配置" rows={[
        { label: '批处理大小', value: settings.batchSize },
        { label: '运行间隔（秒）', value: settings.filterIntervalSec },
        { label: '历史窗口（小时）', value: settings.historyHours },
        { label: '最小历史数据（小时）', value: settings.minHistoryHours },
      ]} />
      <SettingsGroup title="推送器配置" rows={[
        { label: '轮询间隔（秒）', value: settings.pusherIntervalSec },
        { label: '最大重试次数', value: settings.maxRetries },
        { label: '重试基础间隔（秒）', value: settings.retryBaseSec },
      ]} />
      <SettingsGroup title="服务器配置" rows={[
        { label: '监听地址', value: settings.host },
        { label: '端口', value: settings.port },
      ]} />
    </div>
  );
};

const SettingsGroup: React.FC<{ title: string; rows: { label: string; value: string }[] }> = ({ title, rows }) => (
  <div className="settings-group">
    <h4>{title}</h4>
    {rows.map((r, i) => (
      <div key={i} className="setting-row">
        <span className="setting-label">{r.label}</span>
        <span className="setting-value">{r.value}</span>
      </div>
    ))}
  </div>
);

export default SettingsPage;
```

---

## 通用设计模式总结

每个管理页面遵循统一的 CRUD 模式（使用自定义 CSS 组件）：

```
1. 页面加载 → useEffect → fetchData → 表格渲染
2. 添加/编辑 → modal-overlay + form → validateFields → POST/PUT → 刷新列表
3. 删除 → window.confirm() → DELETE → 刷新列表
4. 加载状态 → 显示「加载中...」文字
5. 空状态 → Empty 组件 + 引导操作按钮
6. 错误处理 → API 拦截器自动 Toast
7. 操作反馈 → useToast() hook
```

---

## 预期文件清单

| 文件路径 | 说明 |
|---------|------|
| `src/pages/Sources.tsx` | 数据源管理页（自定义 CSS 组件） |
| `src/pages/Keywords.tsx` | 关键词管理页（自定义 CSS 组件） |
| `src/pages/Channels.tsx` | 推送渠道管理页（自定义 CSS 组件） |
| `src/pages/Tokens.tsx` | Token 管理页（自定义 CSS 组件） |
| `src/pages/Settings.tsx` | 系统设置页（只读配置卡片） |

---

## 验证节点

```bash
cd frontend
npm run dev

# 浏览器访问各页面（确认暗色主题 + 自定义样式正确）：
# http://localhost:5173/sources   → 数据源 CRUD 正常，表格匹配原型列
# http://localhost:5173/keywords  → 关键词 CRUD 正常，状态切换按钮正常
# http://localhost:5173/channels  → 推送渠道 CRUD 正常，测试按钮正常
# http://localhost:5173/tokens    → Token 创建/复制/吊销正常
# http://localhost:5173/settings  → 配置参数只读展示正确

# 测试操作：
# 1. 添加一个 RSS 数据源，确认表格展示匹配原型风格
# 2. 添加一个关键词 "GPT-5"，确认 mono 字体显示
# 3. 添加一个 webhook 推送渠道
# 4. 在 Token 页生成一个新 Token，复制并吊销
# 5. 确认设置页显示 4 组配置卡片

# 编译检查
npm run build
```
