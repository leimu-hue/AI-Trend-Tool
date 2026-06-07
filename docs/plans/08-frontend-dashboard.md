# 步骤 08：前端仪表盘 + 文章日志页 + 响应式布局

## 前置依赖

- 步骤 06-07 已完成（前端脚手架、管理页面、设计 Token、全局样式）
- `docs/Live-Artifact/index.html` 作为视觉参考

## 目标

完成后拥有：
- 热点事件仪表盘（统计卡片 + SVG 趋势图 + 热点表格 + 最新文章）
- 文章日志页（分页 + 过滤 + 处理状态）
- 响应式布局适配（匹配原型的 1024px / 768px 断点）

所有组件使用自定义 CSS 类（`.stats-row`、`.stat-card`、`.panel`、`.badge` 等），保持暗色主题视觉一致性。

---

## 1. 热点仪表盘页 `src/pages/Dashboard.tsx`

### 1.1 页面布局（匹配原型 `view-dashboard`）

从上到下：
1. **统计卡片行**（`.stats-row`）：4 个 `.stat-card` — 数据源、关键词、今日文章、活跃热点
2. **双栏布局**（`.grid-2-wide`）：
   - 左侧（2/3）：关键词趋势图（SVG 折线图，24h）
   - 右侧（1/3）：活跃热点表格
3. **最新文章面板**：表格展示最近 5 篇文章

### 1.2 统计卡片（匹配原型 `.stat-card`）

```tsx
// 4 列网格布局，响应式变为 2 列（≤1024px）或 1 列（≤768px）
<div className="stats-row">
  <div className="stat-card">
    <div className="stat-label">数据源</div>
    <div className="stat-value">{stats.sources}</div>
    <div className="stat-sub">{stats.sourcesDetail}</div>
  </div>
  <div className="stat-card">
    <div className="stat-label">关键词</div>
    <div className="stat-value">{stats.keywords}</div>
    <div className="stat-sub">{stats.keywordsDetail}</div>
  </div>
  <div className="stat-card">
    <div className="stat-label">今日文章</div>
    <div className="stat-value">{stats.todayArticles}</div>
    <div className="stat-sub up">{stats.articleTrend}</div>
  </div>
  <div className="stat-card">
    <div className="stat-label">活跃热点</div>
    <div className="stat-value">{stats.hotspots}</div>
    <div className="stat-sub warn">{stats.hotspotDetail}</div>
  </div>
</div>
```

> `.stat-sub.up` 用 `var(--success)` 绿色，`.stat-sub.warn` 用 `var(--warn)` 黄色。

### 1.3 趋势图（匹配原型 SVG）

原型使用内联 SVG 绘制折线图，包含：
- 渐变填充区域（`linearGradient`，从绿色 25% 透明度到透明）
- 网格背景（`pattern`）
- 折线（`polyline`，`stroke: var(--success)`，`stroke-width: 2`）
- 底部时间标签

**实现方案：使用 ECharts 并配置暗色主题**

```tsx
import ReactECharts from 'echarts-for-react';

const trendChartOption = {
  backgroundColor: 'transparent',
  grid: { left: 40, right: 20, top: 20, bottom: 30 },
  xAxis: {
    type: 'category',
    data: trendLabels,
    axisLabel: { color: 'var(--muted)', fontFamily: 'var(--font-mono)', fontSize: 10 },
    axisLine: { lineStyle: { color: 'rgba(255,255,255,0.05)' } },
  },
  yAxis: {
    type: 'value',
    axisLabel: { color: 'var(--muted)', fontFamily: 'var(--font-mono)', fontSize: 10 },
    splitLine: { lineStyle: { color: 'rgba(255,255,255,0.03)' } },
  },
  series: [{
    type: 'line',
    data: trendData,
    smooth: true,
    lineStyle: { color: '#16a34a', width: 2 },
    areaStyle: {
      color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
        { offset: 0, color: 'rgba(22,163,74,0.25)' },
        { offset: 1, color: 'rgba(22,163,74,0)' },
      ]),
    },
    itemStyle: { color: '#16a34a' },
  }],
  tooltip: { trigger: 'axis' },
};
```

### 1.4 活跃热点表格（匹配原型）

| 列 | 说明 |
|----|------|
| 关键词 | `color: var(--fg)`, `font-weight: 400` |
| 热度 | mono 字体，如「47 次/小时」 |
| 偏离 | `.severity.critical` / `.severity.high` / `.severity.medium` |
| 状态 | badge-success（已推送）/ badge-warn（待推送） |

```tsx
// 偏离度显示 — 匹配原型的 .severity 样式
<span className={`severity ${getSeverityClass(deviation)}`}>
  {deviation.toFixed(1)}σ
</span>
```

### 1.5 最新文章表格（匹配原型）

| 列 | 说明 |
|----|------|
| 来源 | mono 字体，11px |
| 标题 | 链接，truncate 截断（max-width: 320px） |
| 匹配关键词 | mono 字体，绿色 |
| 时间 | mono 字体，11px |
| 状态 | badge-success（已处理） |

右上角有「查看全部 →」按钮跳转到文章日志页。

### 1.6 完整实现结构

```tsx
import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import ReactECharts from 'echarts-for-react';
import client from '../api/client';
import Empty from '../components/Empty';
import { useToast } from '../components/Toast';

const DashboardPage: React.FC = () => {
  const [stats, setStats] = useState({ ... });
  const [hotspots, setHotspots] = useState([]);
  const [articles, setArticles] = useState([]);
  const [trendData, setTrendData] = useState([]);
  const [trendLabels, setTrendLabels] = useState([]);
  const [loading, setLoading] = useState(true);
  const navigate = useNavigate();
  const { showToast } = useToast();

  useEffect(() => {
    // 并行获取统计数据、热点、文章
    Promise.all([
      client.get('/hotspots', { params: { per_page: 50 } }),
      client.get('/articles', { params: { per_page: 5 } }),
      client.get('/sources'),
      client.get('/keywords'),
    ]).then(([hotRes, artRes, srcRes, kwRes]) => {
      // 填充统计卡片数据
      // 填充热点表格
      // 填充最新文章表格
    }).finally(() => setLoading(false));
  }, []);

  // 点击热点行时加载趋势数据
  const handleHotspotClick = async (keywordId: number) => {
    const res = await client.get(`/trend/${keywordId}`, { params: { hours: 24 } });
    setTrendData(res.data.points.map((p: any) => p.count));
    setTrendLabels(res.data.points.map((p: any) => p.hour_bucket));
  };

  if (loading) return <div style={{ color: 'var(--muted)', padding: 48, textAlign: 'center' }}>加载中...</div>;

  return (
    <div>
      {/* 统计卡片 */}
      <div className="stats-row">...</div>

      {/* 双栏：趋势图 + 热点表格 */}
      <div className="grid-2-wide">
        <div className="panel">
          <div className="panel-header">
            <span className="panel-title">关键词趋势（24h）</span>
            <span className="badge badge-success">GPT-5 · 当前热点</span>
          </div>
          <div className="trend-chart">
            <ReactECharts option={trendChartOption} style={{ height: 200 }} />
          </div>
        </div>
        <div className="panel">
          <div className="panel-header">
            <span className="panel-title">活跃热点</span>
            <button className="btn btn-primary btn-sm" onClick={() => {
              client.post('/trigger/filter').then(() => showToast('手动触发过滤器...'));
            }}>手动扫描</button>
          </div>
          <table>...</table>
        </div>
      </div>

      {/* 最新文章 */}
      <div className="panel">
        <div className="panel-header">
          <span className="panel-title">最新文章</span>
          <button className="btn btn-ghost btn-sm" onClick={() => navigate('/articles')}>查看全部 →</button>
        </div>
        <table>...</table>
      </div>
    </div>
  );
};

export default DashboardPage;
```

---

## 2. 文章日志页 `src/pages/Articles.tsx`

### 2.1 页面功能（匹配原型 `view-articles`）

参考原型布局：
- 面板头部：「文章日志」标题 + 「运行过滤器」按钮
- 表格列：#、来源、标题、匹配关键词、发布时间、处理状态
- 客户端分页（或后端分页）

### 2.2 表格列（匹配原型）

| 列 | 说明 |
|----|------|
| # | mono 字体，11px，`color: var(--meta)` |
| 来源 | mono 字体，11px |
| 标题 | 链接，truncate 截断（max-width: 340px） |
| 匹配关键词 | mono 字体，11px，绿色 |
| 发布时间 | mono 字体，11px |
| 处理状态 | badge-success（已处理）/ badge-warn（待处理） |

### 2.3 实现要点

```tsx
import React, { useEffect, useState } from 'react';
import client from '../api/client';
import Empty from '../components/Empty';
import { useToast } from '../components/Toast';

const ArticlesPage: React.FC = () => {
  const [articles, setArticles] = useState([]);
  const [total, setTotal] = useState(0);
  const [page, setPage] = useState(1);
  const [perPage, setPerPage] = useState(20);
  const [sourceFilter, setSourceFilter] = useState<number | undefined>();
  const [processedFilter, setProcessedFilter] = useState<boolean | undefined>();
  const [loading, setLoading] = useState(true);
  const { showToast } = useToast();

  const fetchData = async () => {
    setLoading(true);
    const params: any = { page, per_page: perPage };
    if (sourceFilter !== undefined) params.source_id = sourceFilter;
    if (processedFilter !== undefined) params.processed = processedFilter;
    try {
      const res = await client.get('/articles', { params });
      setArticles(res.data.items || []);
      setTotal(res.data.total || 0);
    } catch {}
    setLoading(false);
  };

  useEffect(() => { fetchData(); }, [page, perPage, sourceFilter, processedFilter]);

  return (
    <div className="panel">
      <div className="panel-header">
        <span className="panel-title">文章日志</span>
        <div className="panel-actions">
          <button className="btn btn-ghost btn-sm" onClick={() => {
            client.post('/trigger/filter').then(() => showToast('过滤器已触发，正在处理...'));
          }}>运行过滤器</button>
        </div>
      </div>
      <div className="table-wrap">
        {loading ? (
          <div style={{ padding: 48, textAlign: 'center', color: 'var(--muted)' }}>加载中...</div>
        ) : articles.length === 0 ? (
          <Empty description="暂无文章" />
        ) : (
          <table>
            <thead>
              <tr><th>#</th><th>来源</th><th>标题</th><th>匹配关键词</th><th>发布时间</th><th>处理状态</th></tr>
            </thead>
            <tbody>
              {articles.map((a, i) => (
                <tr key={a.id}>
                  <td><span className="mono" style={{ fontSize: 11, color: 'var(--meta)' }}>{a.id}</span></td>
                  <td><span className="mono" style={{ fontSize: 11 }}>{a.source_name}</span></td>
                  <td><a href={a.link} target="_blank" className="truncate" style={{ maxWidth: 340 }}>{a.title || '(无标题)'}</a></td>
                  <td><span className="mono" style={{ fontSize: 11, color: 'var(--success)' }}>{a.matched_keywords}</span></td>
                  <td><span className="mono" style={{ fontSize: 11 }}>{a.published_at || '-'}</span></td>
                  <td>
                    <span className={`badge ${a.processed_at ? 'badge-success' : 'badge-warn'}`}>
                      <span className={`badge-dot ${a.processed_at ? 'success-dot' : 'warn-dot'} dot-show`} />
                      {a.processed_at ? '已处理' : '待处理'}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </div>
      {/* 分页控件：可用自定义按钮或简单文字分页 */}
      <div style={{ padding: '12px 18px', display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
        <span className="mono" style={{ fontSize: 11, color: 'var(--muted)' }}>
          共 {total} 条 · 第 {page}/{Math.ceil(total / perPage) || 1} 页
        </span>
        <div style={{ display: 'flex', gap: 8 }}>
          <button className="btn btn-ghost btn-sm" disabled={page <= 1} onClick={() => setPage(p => p - 1)}>上一页</button>
          <button className="btn btn-ghost btn-sm" disabled={page >= Math.ceil(total / perPage)} onClick={() => setPage(p => p + 1)}>下一页</button>
        </div>
      </div>
    </div>
  );
};

export default ArticlesPage;
```

---

## 3. 响应式布局适配

### 3.1 断点规则（匹配原型 CSS）

| 断点 | 屏幕宽度 | 侧边栏 | 统计卡片 | 双栏布局 | 设置网格 |
|------|---------|--------|---------|---------|--------|
| > 1024px | 桌面 | 固定展开 | 4列 | 2列（2:1） | 2列 |
| ≤ 1024px | 平板 | 固定展开 | 2列 | 1列 | 1列 |
| ≤ 768px | 手机 | 隐藏（汉堡菜单） | 1列 | 1列 | 1列 |

### 3.2 响应式 CSS（已在全局样式 global.css 中定义）

```css
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

### 3.3 额外响应式断点（来自 DESIGN-MANIFEST.json）

原型设计支持以下视口矩阵，确保无横向滚动：

| 视口 | 尺寸 | 类别 |
|------|------|------|
| 手机紧凑 | 360×800 | mobile |
| 手机标准 | 390×844 | mobile |
| 手机大 | 430×932 | mobile |
| 折叠屏/小平板 | 600×960 | tablet |
| 平板竖屏 | 820×1180 | tablet |
| 平板横屏 | 1024×768 | desktop |
| 笔记本 | 1366×768 | desktop |
| 桌面 | 1440×900 | desktop |
| 宽屏 | 1920×1080 | wide |

---

## 预期文件清单

| 文件路径 | 说明 |
|---------|------|
| `src/pages/Dashboard.tsx` | 仪表盘页（统计卡片 + ECharts 趋势图 + 热点表格 + 最新文章） |
| `src/pages/Articles.tsx` | 文章日志页（分页 + 过滤 + 自定义表格） |

---

## 验证节点

```bash
cd frontend
npm run dev

# 浏览器访问：
# http://localhost:5173/dashboard
# - 4 个统计卡片正确显示（暗色背景，数字突出）
# - 趋势图显示（ECharts 暗色主题，绿色渐变填充）
# - 热点表格显示（关键词、热度、偏离度、状态徽章）
# - 最新文章表格显示（来源、标题、关键词、时间、状态）
# - 点击热点行更新趋势图
# - 点击「手动扫描」触发过滤器
# - 点击「查看全部」跳转文章日志页

# http://localhost:5173/articles
# - 文章列表分页正常（自定义分页控件）
# - 点击「运行过滤器」触发过滤
# - 表格列匹配原型样式（mono 字体、truncate、badge）

# 响应式测试：
# - 缩小浏览器窗口到 < 1024px：统计卡片变为 2 列，双栏变为单栏
# - 缩小到 < 768px：侧边栏隐藏，统计卡片 1 列，出现汉堡菜单

# 编译检查
npm run build
# 应无 TypeScript 编译错误
```

---

## 端到端完整流程验证

至此，所有前后端步骤完成。进行端到端测试：

```bash
# 1. 启动后端
cd d:/my_project/TrendAITool/trend-monitor
cargo run -- --config config.toml all

# 2. 启动前端
cd d:/my_project/TrendAITool/frontend
npm run dev

# 3. 完整操作流程：
# a. 打开前端 → 输入初始 Token 认证（暗色主题登录页）
# b. 进入仪表盘，确认统计卡片 + 趋势图 + 热点表格 + 最新文章显示正确
# c. 添加 RSS 数据源（如 https://hnrss.org/frontpage）
# d. 添加关键词（如 "AI"、"GPT"）
# e. 添加推送渠道（钉钉 webhook URL）
# f. 等待 Parser 抓取数据（或手动触发 fetch）
# g. 手动触发 Filter（仪表盘「手动扫描」按钮）
# h. 查看仪表盘热点事件和趋势图
# i. 查看文章日志确认数据
# j. 手动触发 Pusher（POST /trigger/pusher）
# k. 确认钉钉收到告警消息
```
