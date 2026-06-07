# 步骤 06：前端脚手架 + 路由 + 认证 + 全局状态管理

## 前置依赖

- 步骤 01-05 已完成（后端 API 全部可用）
- `docs/Live-Artifact/` 目录已就绪（UI 视觉设计源）

## 目标

完成后拥有：
- React + Vite + TypeScript 项目
- 路由结构配置
- API 请求封装（统一 Token 拦截 + 错误处理）
- Token 认证页面
- 全局状态管理（Loading/Error/Empty 组件 + Toast 通知）
- 整体布局框架（侧边栏导航）

---

## 0. 视觉设计源（重要）

**所有前端实现必须以 `docs/Live-Artifact/` 目录中的 HTML 原型为视觉合约。**

| 文件 | 用途 |
|------|------|
| `docs/Live-Artifact/index.html` | 主入口，包含完整 CSS 变量、侧边栏、仪表盘、所有页面视图 |
| `docs/Live-Artifact/data.json` | 模拟数据结构（用于开发阶段的 mock 数据） |
| `docs/Live-Artifact/ai-hotspot-monitor.html` | 独立屏幕文件 |
| `docs/Live-Artifact/template.html` | 模板屏幕文件 |
| `docs/Live-Artifact/DESIGN-HANDOFF.md` | 设计交接说明 |
| `docs/Live-Artifact/DESIGN-MANIFEST.json` | 机器可读的设计清单 |

### 实现原则
- 从原型 CSS 提取设计 Token，再写组件
- 保持暗色主题配色，不使用 Ant Design 默认亮色主题
- 使用自定义 CSS 组件或 Ant Design 的 ConfigProvider 定制暗色主题
- 保留原型中的布局比例、间距节奏、字体层级
- 响应式断点：1024px（平板）、768px（手机）

---

## 1. 项目初始化

```bash
# 在项目根目录下创建前端项目
cd d:/my_project/TrendAITool
npm create vite@latest frontend -- --template react-ts
cd frontend
npm install
```

### 1.1 安装依赖

```bash
# 路由
npm install react-router-dom

# HTTP 客户端
npm install axios

# 图表（ECharts）— 仪表盘趋势图
npm install echarts echarts-for-react

# 日期处理
npm install dayjs
```

> **注意**：不使用 Ant Design。UI 组件基于原型 CSS 自定义实现，保持暗色主题视觉一致性。
> 表单验证可使用原生 HTML5 验证或轻量库如 `react-hook-form` + `zod`（按需引入）。

### 1.2 目录结构

```
frontend/
├── public/
├── src/
│   ├── api/                  # API 请求封装
│   │   ├── client.ts         # axios 实例 + 拦截器
│   │   ├── tokens.ts         # Token API
│   │   ├── sources.ts        # 数据源 API
│   │   ├── keywords.ts       # 关键词 API
│   │   ├── channels.ts       # 推送渠道 API
│   │   └── queries.ts        # 查询 API
│   ├── components/           # 通用组件
│   │   ├── Layout.tsx        # 整体布局（侧边栏 + 内容区）
│   │   ├── Loading.tsx       # 加载状态组件
│   │   ├── Empty.tsx         # 空状态组件
│   │   ├── ErrorBoundary.tsx # 错误边界
│   │   └── Toast.tsx         # Toast 通知封装
│   ├── pages/                # 页面组件
│   │   ├── Auth.tsx          # Token 认证页
│   │   ├── Settings.tsx      # Token 设置页
│   │   ├── Sources.tsx       # 数据源管理
│   │   ├── Keywords.tsx      # 关键词管理
│   │   ├── Channels.tsx      # 推送渠道管理
│   │   ├── Dashboard.tsx     # 热点仪表盘
│   │   └── Articles.tsx      # 文章日志
│   ├── hooks/                # 自定义 hooks
│   │   └── useApi.ts         # 通用 API 调用 hook
│   ├── App.tsx               # 路由配置
│   ├── main.tsx              # 入口
│   └── index.css             # 全局样式
├── index.html
├── package.json
├── tsconfig.json
└── vite.config.ts
```

---

## 2. 设计 Token 系统 `src/styles/tokens.css`

从 `docs/Live-Artifact/index.html` 的 `:root` 提取以下设计 Token 作为全局 CSS 变量：

```css
:root {
  /* 颜色 */
  --bg: #161412;
  --surface: #1f1d1b;
  --fg: #faf9f6;
  --fg-2: #afaeac;
  --muted: #868584;
  --meta: #666469;
  --border: rgba(226, 226, 226, 0.35);
  --accent: #353534;
  --accent-on: #afaeac;
  --accent-hover: #454545;
  --success: #16a34a;
  --warn: #eab308;
  --danger: #dc2626;

  /* 字体 */
  --font-display: "Inter", ui-sans-serif, system-ui, sans-serif;
  --font-body: "Inter", ui-sans-serif, system-ui, sans-serif;
  --font-mono: ui-monospace, "SF Mono", Menlo, Monaco, Consolas, monospace;

  /* 字号 */
  --text-xs: 12px;
  --text-sm: 14px;
  --text-base: 18px;
  --text-lg: 24px;
  --text-xl: 32px;
  --text-2xl: 48px;

  /* 圆角 */
  --radius-sm: 6px;
  --radius-md: 12px;
  --radius-lg: 14px;
  --radius-pill: 9999px;

  /* 阴影/层级 */
  --elev-flat: none;
  --elev-ring: 0 0 0 1px var(--border);
  --elev-raised: 0 5px 15px rgba(0, 0, 0, 0.2);
  --focus-ring: 0 0 0 2px rgba(250, 249, 246, 0.5);

  /* 动效 */
  --motion-fast: 150ms;
  --motion-base: 200ms;
  --ease-standard: cubic-bezier(0.2, 0, 0, 1);

  /* 布局 */
  --container-max: 1500px;
  --sidebar-w: 220px;

  /* 间距 */
  --space-1: 4px;
  --space-2: 8px;
  --space-3: 12px;
  --space-4: 16px;
  --space-5: 20px;
  --space-6: 24px;
  --space-8: 32px;
  --space-12: 48px;
}
```

在 `src/styles/global.css` 中引入并添加基础重置样式（匹配原型的 `box-sizing`, `body` 样式等）。

---

## 3. API 请求封装 `src/api/client.ts`

### 2.1 axios 实例 + Token 拦截器

```typescript
import axios from 'axios';
import { message } from 'antd';

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:8080/api/v1';

const client = axios.create({
  baseURL: API_BASE_URL,
  timeout: 30000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// 请求拦截器：自动携带 Token
client.interceptors.request.use((config) => {
  const token = localStorage.getItem('api_token');
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// 响应拦截器：统一错误处理
client.interceptors.response.use(
  (response) => response,
  (error) => {
    if (error.response) {
      const { status, data } = error.response;

      if (status === 401) {
        // Token 无效或过期，清除并跳转认证页
        localStorage.removeItem('api_token');
        window.location.href = '/auth';
        message.error('认证失效，请重新输入 Token');
      } else {
        // 显示后端返回的错误信息
        const errMsg = data?.error?.message || '请求失败';
        message.error(errMsg);
      }
    } else if (error.request) {
      message.error('网络错误，请检查后端服务是否启动');
    }

    return Promise.reject(error);
  }
);

export default client;
```

### 2.2 API 模块示例 `src/api/tokens.ts`

```typescript
import client from './client';

export interface TokenInfo {
  id: number;
  name: string;
  last_used_at: string | null;
  created_at: string;
  expires_at: string | null;
  revoked: boolean;
}

export interface CreateTokenRequest {
  name: string;
  expires_at?: string;
}

export interface CreateTokenResponse {
  id: number;
  name: string;
  token: string;
  created_at: string;
  expires_at: string | null;
  revoked: boolean;
}

export const tokenApi = {
  list: () => client.get<TokenInfo[]>('/tokens').then(r => r.data),
  create: (data: CreateTokenRequest) =>
    client.post<CreateTokenResponse>('/tokens', data).then(r => r.data),
  revoke: (id: number) => client.delete(`/tokens/${id}`),
};
```

### 2.3 其他 API 模块

`src/api/sources.ts`、`src/api/keywords.ts`、`src/api/channels.ts`、`src/api/queries.ts` 均遵循相同模式，根据后端 API 定义封装对应的 TypeScript 接口和调用函数。

---

## 4. Token 认证页 `src/pages/Auth.tsx`

### 4.1 设计说明

- 全屏暗色背景（`var(--bg)`），居中卡片（`var(--surface)`）
- 卡片样式匹配原型的 `.modal` 样式：`border: 1px solid var(--border)`, `border-radius: var(--radius-lg)`
- 提供输入框让用户粘贴 API Token
- 保存后自动跳转主页
- 若 localStorage 已有 Token，自动验证有效性

### 4.2 实现

```tsx
import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import client from '../api/client';
import '../styles/tokens.css';

const AuthPage: React.FC = () => {
  const [token, setToken] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState('');
  const navigate = useNavigate();

  const handleSubmit = async () => {
    if (!token.trim()) {
      setError('请输入 Token');
      return;
    }
    setLoading(true);
    setError('');
    try {
      localStorage.setItem('api_token', token.trim());
      await client.get('/tokens');
      navigate('/dashboard');
    } catch (e) {
      localStorage.removeItem('api_token');
      setError('Token 无效或已过期');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div style={{
      display: 'flex', justifyContent: 'center', alignItems: 'center',
      minHeight: '100vh', background: 'var(--bg)',
    }}>
      <div className="modal" style={{ minWidth: 420, maxWidth: 520 }}>
        <h3 style={{ color: 'var(--fg)', fontFamily: 'var(--font-display)',
                    fontSize: 'var(--text-lg)', fontWeight: 400, marginBottom: 20 }}>
          AI 热点监控系统
        </h3>
        <p style={{ color: 'var(--muted)', fontFamily: 'var(--font-mono)',
                    fontSize: 'var(--text-xs)', textTransform: 'uppercase',
                    letterSpacing: '0.12em', marginBottom: 16 }}>
          请输入 API Token 以继续
        </p>
        {error && (
          <div style={{ color: 'var(--danger)', border: '1px solid rgba(220,38,38,0.3)',
                       background: 'rgba(220,38,38,0.06)', borderRadius: 'var(--radius-sm)',
                       padding: '8px 12px', fontSize: 13, marginBottom: 14 }}>
            {error}
          </div>
        )}
        <div className="field">
          <label>API Token</label>
          <input
            type="password"
            placeholder="粘贴你的 API Token..."
            value={token}
            onChange={(e) => setToken(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleSubmit()}
          />
        </div>
        <div className="modal-actions">
          <button className="btn btn-primary" onClick={handleSubmit} disabled={loading}>
            {loading ? '验证中...' : '验证并进入'}
          </button>
        </div>
        <p style={{ color: 'var(--meta)', fontSize: 11, marginTop: 16 }}>
          提示：首次启动时，后端日志会输出初始 Token。
        </p>
      </div>
    </div>
  );
};

export default AuthPage;
```

---

## 5. 通用组件

### 5.1 加载状态组件 `src/components/Loading.tsx`

```tsx
import React from 'react';

const Loading: React.FC<{ fullPage?: boolean }> = ({ fullPage }) => (
  <div style={{
    display: 'flex', justifyContent: 'center', alignItems: 'center',
    minHeight: fullPage ? '100vh' : 200,
    color: 'var(--muted)', fontFamily: 'var(--font-mono)', fontSize: 'var(--text-sm)',
  }}>
    <span className="live-dot" style={{ marginRight: 8 }} /> 加载中...
  </div>
);

export default Loading;
```

### 5.2 空状态组件 `src/components/Empty.tsx`

```tsx
import React from 'react';

interface EmptyProps {
  description?: string;
  actionText?: string;
  onAction?: () => void;
}

const Empty: React.FC<EmptyProps> = ({ description = '暂无数据', actionText, onAction }) => (
  <div style={{ textAlign: 'center', padding: '48px 0', color: 'var(--muted)' }}>
    <p style={{ fontFamily: 'var(--font-mono)', fontSize: 'var(--text-sm)' }}>{description}</p>
    {actionText && onAction && (
      <button className="btn btn-primary btn-sm" onClick={onAction} style={{ marginTop: 12 }}>
        {actionText}
      </button>
    )}
  </div>
);

export default Empty;
```

### 5.3 Toast 通知组件 `src/components/Toast.tsx`

匹配原型的 `.toast` 样式（右下角固定定位，暗色面板 + 边框 + 阴影）：

```tsx
import React, { createContext, useContext, useState, useCallback, useRef } from 'react';

interface ToastContextType {
  showToast: (msg: string) => void;
}

const ToastContext = createContext<ToastContextType>({ showToast: () => {} });

export const useToast = () => useContext(ToastContext);

export const ToastProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [msg, setMsg] = useState('');
  const [visible, setVisible] = useState(false);
  const timerRef = useRef<NodeJS.Timeout>();

  const showToast = useCallback((message: string) => {
    setMsg(message);
    setVisible(true);
    if (timerRef.current) clearTimeout(timerRef.current);
    timerRef.current = setTimeout(() => setVisible(false), 2000);
  }, []);

  return (
    <ToastContext.Provider value={{ showToast }}>
      {children}
      <div className={`toast ${visible ? 'show' : ''}`}>{msg}</div>
    </ToastContext.Provider>
  );
};
```

### 5.4 错误边界 `src/components/ErrorBoundary.tsx`

```tsx
import React, { Component, ErrorInfo, ReactNode } from 'react';

interface Props { children: ReactNode; }
interface State { hasError: boolean; error?: Error; }

class ErrorBoundary extends Component<Props, State> {
  state: State = { hasError: false };
  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }
  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('ErrorBoundary:', error, info);
  }
  render() {
    if (this.state.hasError) {
      return (
        <div style={{ textAlign: 'center', padding: 48, color: 'var(--fg)' }}>
          <h3>页面出错了</h3>
          <p style={{ color: 'var(--muted)' }}>{this.state.error?.message}</p>
          <button className="btn btn-primary" onClick={() => window.location.reload()}>
            刷新页面
          </button>
        </div>
      );
    }
    return this.props.children;
  }
}

export default ErrorBoundary;
```

---

## 6. 通用 API Hook `src/hooks/useApi.ts`

管理 API 调用的 loading/error/data 状态：

```typescript
import { useState, useCallback } from 'react';

interface UseApiState<T> {
  data: T | null;
  loading: boolean;
  error: string | null;
}

export function useApi<T>(apiFn: (...args: any[]) => Promise<T>) {
  const [state, setState] = useState<UseApiState<T>>({
    data: null,
    loading: false,
    error: null,
  });

  const execute = useCallback(async (...args: any[]) => {
    setState({ data: null, loading: true, error: null });
    try {
      const data = await apiFn(...args);
      setState({ data, loading: false, error: null });
      return data;
    } catch (e: any) {
      const error = e?.response?.data?.error?.message || e.message || '请求失败';
      setState({ data: null, loading: false, error });
      throw e;
    }
  }, []);

  return { ...state, execute };
}
```

---

## 7. 布局组件 `src/components/Layout.tsx`

**严格匹配原型 `docs/Live-Artifact/index.html` 中的侧边栏 + 顶栏布局。**

原型结构：
- **侧边栏**（`--sidebar-w: 220px`）：品牌区 → 导航分组（监控/配置/系统）→ 底部 LIVE 状态
- **顶栏**：页面标题 + BETA 徽章 + 时间戳
- **主内容区**：`overflow-y: auto`, `padding: 24px`

```tsx
import React, { useState, useEffect } from 'react';
import { Outlet, useNavigate, useLocation } from 'react-router-dom';
import '../styles/tokens.css';
import '../styles/global.css';

const navItems = [
  { section: '监控' },
  { key: '/dashboard', label: '仪表盘', icon: 'dashboard' },
  { key: '/articles',  label: '文章日志', icon: 'articles' },
  { section: '配置' },
  { key: '/sources',   label: '数据源', icon: 'sources' },
  { key: '/keywords',  label: '关键词', icon: 'keywords' },
  { key: '/channels',  label: '推送渠道', icon: 'channels' },
  { key: '/tokens',    label: 'API 令牌', icon: 'tokens' },
  { section: '系统' },
  { key: '/settings',  label: '设置', icon: 'settings' },
];

const viewTitles: Record<string, string> = {
  '/dashboard': '仪表盘',
  '/articles': '文章日志',
  '/sources': '数据源管理',
  '/keywords': '关键词管理',
  '/channels': '推送渠道管理',
  '/tokens': 'API 令牌管理',
  '/settings': '系统设置',
};

const AppLayout: React.FC = () => {
  const [sidebarOpen, setSidebarOpen] = useState(false);
  const navigate = useNavigate();
  const location = useLocation();
  const currentTitle = viewTitles[location.pathname] || '';

  // 响应式：768px 以下自动关闭侧边栏
  useEffect(() => {
    const handleResize = () => {
      if (window.innerWidth > 768) setSidebarOpen(false);
    };
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  return (
    <>
      {/* 侧边栏 */}
      <aside className={`sidebar ${sidebarOpen ? 'open' : ''}`}>
        <div className="sidebar-brand">
          <div className="logo-row">
            <div className="logo-icon">◈</div>
            <div>
              <div className="brand-name">AI 热点监控</div>
              <div className="brand-sub">多源实时监控 · 关键词告警 · 趋势分析</div>
            </div>
          </div>
        </div>
        <nav className="sidebar-nav">
          {navItems.map((item, i) =>
            'section' in item ? (
              <div key={i} className="nav-section">{item.section}</div>
            ) : (
              <button
                key={item.key}
                className={`nav-item ${location.pathname === item.key ? 'active' : ''}`}
                onClick={() => { navigate(item.key!); setSidebarOpen(false); }}
              >
                {/* SVG 图标从原型 HTML 中复制对应 icon */}
                {item.label}
              </button>
            )
          )}
        </nav>
        <div className="sidebar-footer">
          <div className="live-dot">LIVE</div>
          <div className="sync-text">监控中 · 每5分钟自动刷新</div>
        </div>
      </aside>

      {/* 主区域 */}
      <div className="main-wrap">
        <header className="topbar">
          <div className="topbar-left">
            <button className="menu-btn" onClick={() => setSidebarOpen(!sidebarOpen)}>
              ☰
            </button>
            <span className="topbar-title">{currentTitle}</span>
            <span className="topbar-badge">BETA</span>
          </div>
          <div className="topbar-right">
            <span className="mono" style={{ fontSize: 11, color: 'var(--muted)' }}>
              {new Date().toISOString().replace('T', ' ').slice(0, 19)} UTC
            </span>
          </div>
        </header>
        <main className="main-content">
          <Outlet />
        </main>
      </div>
    </>
  );
};

export default AppLayout;
```

---

## 8. 全局样式 `src/styles/global.css`

从原型提取的全局样式，包含按钮、表格、徽章、面板、弹窗等基础组件类：

```css
/* 导入 tokens */
@import './tokens.css';

*, *::before, *::after { box-sizing: border-box; }
html, body { margin: 0; padding: 0; height: 100%; }
body {
  background: var(--bg); color: var(--fg-2);
  font-family: var(--font-body); font-size: 14px; line-height: 1.5;
  -webkit-font-smoothing: antialiased; display: flex; overflow: hidden;
}
a { color: var(--fg); text-decoration: underline; text-decoration-color: var(--meta); text-underline-offset: 4px; }
a:hover { text-decoration-color: var(--fg); }
code, .mono { font-family: var(--font-mono); }

/* 按钮 */
.btn {
  display: inline-flex; align-items: center; gap: 6px;
  padding: 8px 16px; border-radius: var(--radius-sm);
  font-family: var(--font-body); font-size: 13px; font-weight: 400;
  cursor: pointer; border: none; line-height: 1.3;
  transition: background var(--motion-fast) var(--ease-standard);
}
.btn:focus-visible { outline: none; box-shadow: var(--focus-ring); }
.btn-primary { background: var(--accent); color: var(--accent-on); }
.btn-primary:hover { background: var(--accent-hover); color: var(--fg); }
.btn-ghost { background: transparent; color: var(--fg-2); }
.btn-ghost:hover { background: rgba(255,255,255,0.04); color: var(--fg); }
.btn-sm { padding: 5px 10px; font-size: 11.5px; }
.btn-danger { background: transparent; color: var(--danger); border: 1px solid rgba(220,38,38,0.3); }
.btn-danger:hover { background: rgba(220,38,38,0.1); }

/* 面板 */
.panel { background: var(--surface); border: 1px solid var(--border); border-radius: var(--radius-md); overflow: hidden; margin-bottom: 18px; }
.panel-header { display: flex; align-items: center; justify-content: space-between; padding: 14px 18px; border-bottom: 1px solid var(--border); }
.panel-title { font-family: var(--font-display); font-size: 15px; color: var(--fg); font-weight: 400; }

/* 表格 */
.table-wrap { overflow-x: auto; }
table { width: 100%; border-collapse: collapse; }
th, td { padding: 10px 14px; text-align: left; font-size: 13px; border-bottom: 1px solid var(--border); white-space: nowrap; }
th { font-family: var(--font-mono); font-size: 10px; color: var(--muted); text-transform: uppercase; letter-spacing: 0.1em; font-weight: 400; background: rgba(255,255,255,0.015); }
td { color: var(--fg-2); }
tr:hover td { background: rgba(255,255,255,0.015); }

/* 徽章 */
.badge { display: inline-flex; align-items: center; gap: 5px; padding: 3px 10px; border-radius: var(--radius-pill); font-family: var(--font-mono); font-size: 10.5px; white-space: nowrap; }
.badge-success { color: var(--success); border: 1px solid rgba(22,163,74,0.3); background: rgba(22,163,74,0.06); }
.badge-warn { color: var(--warn); border: 1px solid rgba(234,179,8,0.3); background: rgba(234,179,8,0.06); }
.badge-danger { color: var(--danger); border: 1px solid rgba(220,38,38,0.3); background: rgba(220,38,38,0.06); }
.badge-neutral { color: var(--muted); border: 1px solid var(--border); }

/* 弹窗 */
.modal-overlay {
  display: none; position: fixed; inset: 0; z-index: 100;
  background: rgba(0,0,0,0.6); backdrop-filter: blur(4px);
  align-items: center; justify-content: center;
}
.modal-overlay.open { display: flex; }
.modal {
  background: var(--surface); border: 1px solid var(--border);
  border-radius: var(--radius-lg); padding: 24px;
  min-width: 420px; max-width: 520px; width: 90%;
  box-shadow: var(--elev-raised);
}
.modal h3 { font-family: var(--font-display); font-size: var(--text-lg); font-weight: 400; color: var(--fg); margin: 0 0 20px; }
.field { display: flex; flex-direction: column; gap: 6px; margin-bottom: 14px; }
.field label { font-size: 12px; color: var(--fg-2); font-family: var(--font-mono); text-transform: uppercase; letter-spacing: 0.08em; }
.field input, .field select {
  padding: 10px 12px; border-radius: var(--radius-sm);
  border: 1px solid var(--border); background: var(--bg);
  color: var(--fg); font-family: var(--font-body); font-size: 14px;
  outline: none; transition: border-color var(--motion-fast);
}
.field input:focus, .field select:focus { border-color: var(--fg); box-shadow: var(--focus-ring); }
.modal-actions { display: flex; gap: 10px; justify-content: flex-end; margin-top: 20px; }
.field-help { font-size: 11px; color: var(--meta); }

/* Toast */
.toast {
  position: fixed; bottom: 24px; right: 24px; z-index: 200;
  background: var(--surface); border: 1px solid var(--border);
  border-radius: var(--radius-md); padding: 12px 18px;
  font-size: 13px; color: var(--fg); box-shadow: var(--elev-raised);
  display: none; align-items: center; gap: 8px;
}
.toast.show { display: flex; animation: toastIn 0.3s var(--ease-standard); }
@keyframes toastIn { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }

/* 截断 */
.truncate { display: inline-block; vertical-align: middle; max-width: 240px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

/* 响应式 */
@media (max-width: 1024px) {
  .stats-row { grid-template-columns: repeat(2, 1fr); }
  .grid-2, .grid-2-wide { grid-template-columns: 1fr; }
  .settings-grid { grid-template-columns: 1fr; }
}
@media (max-width: 768px) {
  .sidebar { position: fixed; left: 0; top: 0; transform: translateX(-100%); z-index: 50; }
  .sidebar.open { transform: translateX(0); }
  .topbar .menu-btn { display: block; }
  .stats-row { grid-template-columns: 1fr; }
  .modal { min-width: unset; padding: 20px; }
}
```

---

## 9. 路由配置 `src/App.tsx`

```tsx
import React, { useEffect, useState } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import AppLayout from './components/Layout';
import ErrorBoundary from './components/ErrorBoundary';
import AuthPage from './pages/Auth';
import Dashboard from './pages/Dashboard';
import Sources from './pages/Sources';
import Keywords from './pages/Keywords';
import Channels from './pages/Channels';
import Articles from './pages/Articles';
import Settings from './pages/Settings';
import Loading from './components/Loading';

const ProtectedRoute: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const token = localStorage.getItem('api_token');
  if (!token) {
    return <Navigate to="/auth" replace />;
  }
  return <>{children}</>;
};

const App: React.FC = () => {
  return (
    <ErrorBoundary>
      <BrowserRouter>
        <Routes>
          <Route path="/auth" element={<AuthPage />} />
          <Route
            path="/"
            element={
              <ProtectedRoute>
                <AppLayout />
              </ProtectedRoute>
            }
          >
            <Route index element={<Navigate to="/dashboard" replace />} />
            <Route path="dashboard" element={<Dashboard />} />
            <Route path="sources" element={<Sources />} />
            <Route path="keywords" element={<Keywords />} />
            <Route path="channels" element={<Channels />} />
            <Route path="articles" element={<Articles />} />
            <Route path="settings" element={<Settings />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </ErrorBoundary>
  );
};

export default App;
```

---

## 10. 环境变量

创建 `frontend/.env.development`：

```
VITE_API_BASE_URL=http://localhost:8080/api/v1
```

---

## 预期文件清单

| 文件路径 | 说明 |
|---------|------|
| `frontend/` | Vite + React + TS 项目 |
| `src/styles/tokens.css` | 设计 Token（从原型 :root 提取） |
| `src/styles/global.css` | 全局样式（按钮、面板、表格、弹窗等） |
| `src/api/client.ts` | axios 实例 + Token/错误拦截器 |
| `src/api/tokens.ts` | Token API 封装 |
| `src/api/sources.ts` | 数据源 API 封装 |
| `src/api/keywords.ts` | 关键词 API 封装 |
| `src/api/channels.ts` | 推送渠道 API 封装 |
| `src/api/queries.ts` | 查询 API 封装 |
| `src/components/Layout.tsx` | 侧边栏布局（匹配原型） |
| `src/components/Loading.tsx` | 加载状态组件 |
| `src/components/Empty.tsx` | 空状态组件 |
| `src/components/Toast.tsx` | Toast 通知（Context + Provider） |
| `src/components/ErrorBoundary.tsx` | 错误边界 |
| `src/hooks/useApi.ts` | API 状态管理 hook |
| `src/pages/Auth.tsx` | Token 认证页（暗色主题） |
| `src/App.tsx` | 路由配置 |
| `src/main.tsx` | 入口（包裹 ToastProvider） |
| `.env.development` | API 基地址 |

---

## 验证节点

```bash
cd frontend
npm install
npm run dev

# 打开浏览器访问 http://localhost:5173
# 应跳转到 /auth 认证页（暗色主题，居中卡片）
# 输入有效 Token 后跳转到 /dashboard（侧边栏 + 顶栏布局正确）
# 侧边栏导航分组正确：监控（仪表盘、文章日志）、配置（数据源、关键词、推送渠道、API令牌）、系统（设置）

# 编译检查
npm run build
# 应无 TypeScript 编译错误
```
