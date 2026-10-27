/** 查找高亮工具：按查询词切分文本、统计树中命中数 */

export interface Segment {
  t: string
  hit: boolean
}

/** 按查找词切分文本，返回带命中标记的片段（大小写不敏感可选） */
export function splitSegments(text: string, query: string, caseSensitive: boolean): Segment[] {
  const q = query.trim()
  if (!q) return [{ t: text, hit: false }]
  const hay = caseSensitive ? text : text.toLowerCase()
  const needle = caseSensitive ? q : q.toLowerCase()
  const segs: Segment[] = []
  let i = 0
  let j = hay.indexOf(needle)
  while (j !== -1) {
    if (j > i) segs.push({ t: text.slice(i, j), hit: false })
    segs.push({ t: text.slice(j, j + needle.length), hit: true })
    i = j + needle.length
    j = hay.indexOf(needle, i)
  }
  if (i < text.length) segs.push({ t: text.slice(i), hit: false })
  return segs
}

export function countSegments(text: string, query: string, caseSensitive: boolean): number {
  const q = query.trim()
  if (!q) return 0
  return splitSegments(text, q, caseSensitive).filter((s) => s.hit).length
}

/** 标量的渲染文本（与 JsonNode 展示一致：字符串带引号，其余 String()） */
function renderScalar(v: unknown): string {
  return typeof v === 'string' ? JSON.stringify(v) : String(v)
}

/** 递归统计树中 key / 标量值的命中次数（按渲染文本匹配） */
export function countTreeMatches(node: unknown, query: string, caseSensitive: boolean): number {
  const q = query.trim()
  if (!q) return 0
  let n = 0
  const walk = (v: unknown) => {
    if (v === null || typeof v !== 'object') {
      n += countSegments(renderScalar(v), q, caseSensitive)
      return
    }
    if (Array.isArray(v)) {
      v.forEach(walk)
      return
    }
    for (const [k, val] of Object.entries(v)) {
      n += countSegments(JSON.stringify(k), q, caseSensitive)
      walk(val)
    }
  }
  walk(node)
  return n
}
