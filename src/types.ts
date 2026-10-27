export type Indent = 'twoSpaces' | 'fourSpaces' | 'tab' | 'minify'

/** 与 Rust 端 JsonOptions（camelCase serde）一一对应 */
export interface JsonOptions {
  indent: Indent
  sortKeys: boolean
  allowTrailingCommas: boolean
  allowComments: boolean
  autoRepair: boolean
  /** 容错：清理不可见控制字符（字符串外剔除、字符串内转义） */
  stripControlChars: boolean
}

export interface JsonErrorInfo {
  message: string
  line: number
  column: number
  category: string
}

export interface JsonStats {
  bytesIn: number
  bytesOut: number
  keys: number
  depth: number
}

/** 与 Rust 端 JsonOutcome（camelCase serde）一一对应 */
export interface JsonOutcome {
  ok: boolean
  empty: boolean
  output: string | null
  error: JsonErrorInfo | null
  repaired: boolean
  warnings: string[]
  stats: JsonStats | null
  /** 解析后的结构，树形渲染用 */
  value: unknown | null
}
