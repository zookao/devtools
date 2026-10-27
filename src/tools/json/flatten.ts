/** 树形 JSON 拍平为行数组：每行一个块级 div，缩进 = level × indent，彻底规避 block-in-inline 布局歧义 */

import { splitSegments } from './highlight'

export interface FSegment {
  t: string
  hit?: boolean
  cls?: string
}

export interface FlatLine {
  /** 缩进级别（0 起） */
  level: number
  /** 节点路径（折叠状态键），闭合行为 `${path}#close` */
  path: string
  /** 对象子项的 key 片段（数组子项无） */
  keySegs?: FSegment[]
  /** 容器行的折叠开关 */
  toggle?: { bracket: string; collapsed: boolean }
  /** 折叠时的预览摘要 */
  preview?: string
  /** 值 / 预览 / 闭合符片段 */
  value: FSegment[]
  /** 行尾逗号 */
  comma: boolean
}

function scalarCls(v: unknown): string {
  if (v === null) return 'j-null'
  if (typeof v === 'string') return 'j-string'
  if (typeof v === 'number') return 'j-number'
  if (typeof v === 'boolean') return 'j-boolean'
  return ''
}

function scalarText(v: unknown): string {
  if (v === null) return 'null'
  if (typeof v === 'string') return JSON.stringify(v)
  return String(v)
}

function previewScalar(v: unknown): string {
  if (v === null) return 'null'
  if (typeof v === 'string') {
    return JSON.stringify(v.length > 12 ? v.slice(0, 12) + '…' : v)
  }
  if (typeof v === 'object') return Array.isArray(v) ? '[…]' : '{…}'
  return String(v)
}

function previewOf(v: unknown): string {
  if (Array.isArray(v)) {
    const items = v.slice(0, 3).map(previewScalar)
    return `[${items.join(', ')}${v.length > 3 ? ', …' : ''}]`
  }
  const keys = Object.keys(v as object)
  const items = keys.slice(0, 3).map((k) => `${k}: ${previewScalar((v as Record<string, unknown>)[k])}`)
  return `{${items.join(', ')}${keys.length > 3 ? ', …' : ''}}`
}

export interface FlattenOpts {
  query: string
  caseSensitive: boolean
  /** 用户折叠的节点路径集合（查找模式下强制展开，此集合不生效） */
  collapsed: Set<string>
}

export function flatten(node: unknown, opts: FlattenOpts): FlatLine[] {
  const lines: FlatLine[] = []
  const q = opts.query.trim()
  const cs = opts.caseSensitive

  const seg = (text: string, cls?: string): FSegment[] =>
    splitSegments(text, q, cs).map((s) => (s.hit ? { ...s, cls } : { t: s.t, cls }))

  const walk = (
    v: unknown,
    path: string,
    level: number,
    isLast: boolean,
    keySegs: FSegment[] | undefined,
  ) => {
    const isObj = typeof v === 'object' && v !== null && !Array.isArray(v)
    const isArr = Array.isArray(v)

    // 标量
    if (!isObj && !isArr) {
      lines.push({
        level,
        path,
        keySegs,
        value: seg(scalarText(v), scalarCls(v)),
        comma: !isLast,
      })
      return
    }

    const bracket = isObj ? '{' : '['
    const close = isObj ? '}' : ']'
    const entries: Array<[string, unknown]> = isObj
      ? Object.entries(v as object)
      : (v as unknown[]).map((item, i) => [String(i), item])

    // 空容器
    if (entries.length === 0) {
      lines.push({
        level,
        path,
        keySegs,
        value: [{ t: bracket + close, cls: 'j-punct' }],
        comma: !isLast,
      })
      return
    }

    // 查找模式强制展开；否则读用户折叠状态
    const collapsed = !q && opts.collapsed.has(path)

    lines.push({
      level,
      path,
      keySegs,
      toggle: { bracket, collapsed },
      preview: collapsed ? previewOf(v) : undefined,
      value: [],
      // 展开时逗号由闭合行携带；折叠时本行行尾带逗号
      comma: collapsed ? !isLast : false,
    })

    if (collapsed) return

    entries.forEach(([k, child], i) => {
      const childKey = isObj ? seg(JSON.stringify(k), 'j-key') : undefined
      walk(child, `${path}/${k}`, level + 1, i === entries.length - 1, childKey)
    })

    lines.push({
      level,
      path: `${path}#close`,
      value: [{ t: close, cls: 'j-punct' }],
      comma: !isLast,
    })
  }

  walk(node, '', 0, true, undefined)
  return lines
}
