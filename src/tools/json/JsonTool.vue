<script setup lang="ts">
import { computed, nextTick, onActivated, onDeactivated, reactive, ref, watch } from 'vue'
import Icon from '../../components/Icon.vue'
import FindBar from '../../components/FindBar.vue'
import { clipboardReadText, clipboardWriteText, jsonProcess } from '../../api'
import type { Indent, JsonOptions, JsonOutcome } from '../../types'
import { flatten } from './flatten'
import { countSegments, countTreeMatches, splitSegments } from './highlight'
import { SAMPLE } from './sample'

/** UI 选项（localStorage 持久化）。注释与尾随逗号在 UI 合并为一个开关 */
interface UiOptions {
  allowTrailingCommasAndComments: boolean
  autoRepair: boolean
  sortKeys: boolean
  stripControlChars: boolean
  indent: Indent
}

const DEFAULT_OPTIONS: UiOptions = {
  allowTrailingCommasAndComments: false,
  autoRepair: false,
  sortKeys: false,
  stripControlChars: false,
  indent: 'twoSpaces',
}

const OPTIONS_KEY = 'devtools:json:options'
const SPLIT_KEY = 'devtools:json:split'

function loadOptions(): UiOptions {
  try {
    const raw = localStorage.getItem(OPTIONS_KEY)
    if (raw) return { ...DEFAULT_OPTIONS, ...JSON.parse(raw) }
  } catch {
    /* ignore */
  }
  return { ...DEFAULT_OPTIONS }
}

const options = reactive(loadOptions())
watch(
  options,
  () => localStorage.setItem(OPTIONS_KEY, JSON.stringify(options)),
  { deep: true },
)

// ---------- 状态 ----------

const input = ref('')
const outcome = ref<JsonOutcome | null>(null)
const showOptions = ref(false)
const copied = ref(false)
const clipboardError = ref('')
/** 用户折叠的节点路径集合（重新格式化时重置；查找模式强制展开） */
const collapsedPaths = ref<Set<string>>(new Set())

// ---------- 查找（两栏独立，⌘F/Ctrl+F 打开最近操作面板的查找栏） ----------

const showFindInput = ref(false)
const showFindOutput = ref(false)
const findInputQuery = ref('')
const findOutputQuery = ref('')
const inputFindIndex = ref(0)
const caseSensitive = ref(false)
const lastPane = ref<'input' | 'output'>('input')

const editorEl = ref<HTMLTextAreaElement | null>(null)
const gutterEl = ref<HTMLElement | null>(null)

// ---------- 行号（与编辑器同步滚动） ----------

const lineCount = computed(() => input.value.split('\n').length)

function syncGutter() {
  const g = gutterEl.value
  const ta = editorEl.value
  if (g && ta) g.scrollTop = ta.scrollTop
}

const findBarInput = ref<InstanceType<typeof FindBar> | null>(null)
const findBarOutput = ref<InstanceType<typeof FindBar> | null>(null)

/** Input（textarea）中的全部匹配 [start, end) */
const inputMatches = computed<Array<[number, number]>>(() => {
  const q = findInputQuery.value.trim()
  if (!q) return []
  const hay = caseSensitive.value ? input.value : input.value.toLowerCase()
  const needle = caseSensitive.value ? q : q.toLowerCase()
  const out: Array<[number, number]> = []
  let i = hay.indexOf(needle)
  while (i !== -1) {
    out.push([i, i + needle.length])
    i = hay.indexOf(needle, i + needle.length)
  }
  return out
})

watch(findInputQuery, () => {
  inputFindIndex.value = 0
})

/** 跳转并原生选中 Input 中第 n 个匹配（浏览器自动滚动到选区） */
function gotoInputMatch(dir: 1 | -1) {
  const ms = inputMatches.value
  if (!ms.length) return
  inputFindIndex.value = (inputFindIndex.value + dir + ms.length) % ms.length
  const [s, t] = ms[inputFindIndex.value]
  editorEl.value?.focus()
  editorEl.value?.setSelectionRange(s, t)
}

/** Output 命中数：树模式统计 key/标量渲染文本；Minify 模式统计纯文本 */
const outputMatches = computed(() => {
  const q = findOutputQuery.value.trim()
  if (!q) return 0
  if (options.indent === 'minify') {
    return countSegments(outputText.value, q, caseSensitive.value)
  }
  return treeData.value ? countTreeMatches(treeData.value, q, caseSensitive.value) : 0
})

/** 查找关闭时高亮立即消失（树收到的 query 置空） */
const outputQuery = computed(() => (showFindOutput.value ? findOutputQuery.value : ''))

/** Minify 纯文本高亮片段 */
const outputTextSegs = computed(() =>
  splitSegments(outputText.value, outputQuery.value.trim(), caseSensitive.value),
)

/** 树拍平：每行一个块级 div，缩进 = level × indent（数值精确，无嵌套歧义） */
const flatLines = computed(() => {
  if (!hasTree.value || treeData.value == null) return []
  return flatten(treeData.value, {
    query: outputQuery.value,
    caseSensitive: caseSensitive.value,
    collapsed: collapsedPaths.value,
  })
})

function toggleNode(path: string) {
  const next = new Set(collapsedPaths.value)
  if (next.has(path)) next.delete(path)
  else next.add(path)
  collapsedPaths.value = next
}

function closeFindInput() {
  showFindInput.value = false
  findInputQuery.value = ''
}

function closeFindOutput() {
  showFindOutput.value = false
  findOutputQuery.value = ''
}

function openFind(pane: 'input' | 'output') {
  if (pane === 'input') {
    showFindInput.value = true
    nextTick(() => findBarInput.value?.focus())
  } else {
    showFindOutput.value = true
    nextTick(() => findBarOutput.value?.focus())
  }
}

// ---------- 构建 Rust 选项 ----------

function buildOpts(): JsonOptions {
  return {
    indent: options.indent,
    sortKeys: options.sortKeys,
    allowComments: options.allowTrailingCommasAndComments,
    allowTrailingCommas: options.allowTrailingCommasAndComments,
    autoRepair: options.autoRepair,
    stripControlChars: options.stripControlChars,
  }
}

// ---------- 执行 ----------

async function run() {
  if (!input.value.trim()) {
    outcome.value = null
    return
  }
  outcome.value = await jsonProcess(input.value, buildOpts())
  collapsedPaths.value = new Set()
}

// 选项或缩进变化时，若已执行过则自动刷新结果
watch(
  () => JSON.stringify(options),
  () => {
    if (outcome.value !== null && input.value.trim()) run()
  },
)

// ---------- 工具按钮 ----------

async function pasteFromClipboard() {
  clipboardError.value = ''
  try {
    const text = await clipboardReadText()
    if (text) {
      input.value = text
      await run()
    }
  } catch (e) {
    clipboardError.value = String(e)
  }
}

async function fillSample() {
  input.value = SAMPLE
  await run()
}

function clearAll() {
  input.value = ''
  outcome.value = null
}

async function copyOutput() {
  if (!outcome.value?.output) return
  try {
    await clipboardWriteText(outcome.value.output)
    copied.value = true
    setTimeout(() => (copied.value = false), 1500)
  } catch {
    /* ignore */
  }
}

function resetOptions() {
  Object.assign(options, DEFAULT_OPTIONS)
}

// ---------- 选项弹层：点击外部关闭 ----------

const optionsAnchor = ref<HTMLElement | null>(null)

function onDocClick(e: MouseEvent) {
  if (showOptions.value && optionsAnchor.value && !optionsAnchor.value.contains(e.target as Node)) {
    showOptions.value = false
  }
}

// ---------- 左右分栏拖拽 ----------

const splitEl = ref<HTMLElement | null>(null)
const leftRatio = ref(clampRatio(parseFloat(localStorage.getItem(SPLIT_KEY) || '0.5')))

function clampRatio(r: number) {
  if (!Number.isFinite(r)) return 0.5
  return Math.min(0.85, Math.max(0.15, r))
}

const gridStyle = computed(() => ({
  gridTemplateColumns: `${leftRatio.value}fr 6px ${1 - leftRatio.value}fr`,
}))

function startDrag(e: PointerEvent) {
  e.preventDefault()
  const el = splitEl.value
  if (!el) return
  const rect = el.getBoundingClientRect()
  const onMove = (ev: PointerEvent) => {
    leftRatio.value = clampRatio((ev.clientX - rect.left) / rect.width)
  }
  const onUp = () => {
    window.removeEventListener('pointermove', onMove)
    window.removeEventListener('pointerup', onUp)
    window.removeEventListener('pointercancel', onUp)
    window.removeEventListener('blur', onUp)
    localStorage.setItem(SPLIT_KEY, String(leftRatio.value))
  }
  window.addEventListener('pointermove', onMove)
  window.addEventListener('pointerup', onUp)
  // 拖到窗口外松开或窗口失焦时收不到 pointerup，兜底移除监听避免残留
  window.addEventListener('pointercancel', onUp)
  window.addEventListener('blur', onUp)
}

function onDividerKeydown(e: KeyboardEvent) {
  const step = 0.02
  if (e.key === 'ArrowLeft') {
    leftRatio.value = clampRatio(leftRatio.value - step)
    e.preventDefault()
  } else if (e.key === 'ArrowRight') {
    leftRatio.value = clampRatio(leftRatio.value + step)
    e.preventDefault()
  }
}

// ---------- 快捷键与编辑器体验 ----------

function onGlobalKeydown(e: KeyboardEvent) {
  const mod = e.metaKey || e.ctrlKey
  const key = e.key.toLowerCase()
  if (mod && e.shiftKey && key === 'f') {
    e.preventDefault()
    run()
  } else if (mod && !e.shiftKey && key === 'f') {
    e.preventDefault()
    openFind(lastPane.value)
  }
}

/** textarea 中按 Tab 插入两个空格而非失焦 */
function onEditorKeydown(e: KeyboardEvent) {
  if (e.key === 'Tab') {
    e.preventDefault()
    const ta = e.target as HTMLTextAreaElement
    const { selectionStart: s, selectionEnd: en } = ta
    input.value = ta.value.slice(0, s) + '  ' + ta.value.slice(en)
    nextTick(() => {
      ta.selectionStart = ta.selectionEnd = s + 2
    })
  }
}

/** 粘贴后自动格式化。paste 事件先于文本插入与 v-model 同步，故用宏任务 */
function onEditorPaste() {
  setTimeout(() => run(), 0)
}

// 被 KeepAlive 缓存时不会触发 unmount，全局监听只能在停用/激活时摘挂
onActivated(() => {
  document.addEventListener('click', onDocClick)
  window.addEventListener('keydown', onGlobalKeydown)
})
onDeactivated(() => {
  document.removeEventListener('click', onDocClick)
  window.removeEventListener('keydown', onGlobalKeydown)
})

// ---------- 状态栏 ----------

const status = computed(() => {
  const o = outcome.value
  if (!o) {
    return {
      cls: '',
      hasResult: false,
      text: 'Paste or type JSON, then press ⌘⇧F or click the ⚡ button.',
    }
  }
  if (o.empty) {
    return { cls: '', hasResult: false, text: 'Input is empty.' }
  }
  if (!o.ok) {
    const e = o.error
    return {
      cls: 'err',
      hasResult: false,
      text: e ? `Invalid: line ${e.line}, column ${e.column} — ${e.message}` : 'Invalid JSON.',
    }
  }
  const parts = ['Valid JSON']
  if (o.repaired) parts.push('Auto-repaired')
  return { cls: 'ok', hasResult: true, text: parts.join(' · ') }
})

const statsText = computed(() => {
  const s = outcome.value?.stats
  if (!s || !outcome.value?.ok) return ''
  const fmt = (n: number) => (n >= 1024 ? `${(n / 1024).toFixed(1)} KB` : `${n} B`)
  return `${s.keys} keys · depth ${s.depth} · ${fmt(s.bytesIn)} → ${fmt(s.bytesOut)}`
})

const warningsText = computed(() => outcome.value?.warnings.slice(1).join(' ') || '')

/** 树形视图的层级缩进宽度，跟随缩进选项（与编辑器 Tab=2 空格保持一致） */
const treeIndent = computed(() => (options.indent === 'fourSpaces' ? '4ch' : '2ch'))

/** Minify 时展示纯文本单行，其余展示可折叠树 */
const hasTree = computed(
  () =>
    outcome.value?.ok === true &&
    outcome.value.value != null &&
    options.indent !== 'minify',
)
const treeData = computed(() => outcome.value?.value ?? null)
const outputText = computed(() => outcome.value?.output ?? '')
</script>

<template>
  <div class="tool-body">
    <div ref="splitEl" class="split" :style="gridStyle">
      <!-- Input（左） -->
      <section class="pane" @pointerdown="lastPane = 'input'">
        <div class="pane-header">
          <span class="pane-label">Input:</span>
          <button class="icon-btn primary" title="Format now (⌘⇧F)" @click="run()">
            <Icon name="bolt" />
          </button>
          <button class="icon-btn" title="Paste from clipboard" @click="pasteFromClipboard()">
            <Icon name="clipboard" />
          </button>
          <button class="icon-btn" title="Insert sample" @click="fillSample()">Sample</button>
          <button class="icon-btn" title="Clear" @click="clearAll()">
            <Icon name="x" />
          </button>
          <span class="spacer" />
          <div ref="optionsAnchor" class="popover-anchor">
            <button class="icon-btn" title="Options" @click="showOptions = !showOptions">
              <Icon name="sliders" />
            </button>
            <div v-if="showOptions" class="popover" @click.stop>
              <label class="opt-row">
                <input v-model="options.allowTrailingCommasAndComments" type="checkbox" />
                <span>Allow trailing commas and comments in JSON</span>
              </label>
              <label class="opt-row">
                <input v-model="options.autoRepair" type="checkbox" />
                <span>Auto repair invalid JSON if possible (e.g. quote bare keys)</span>
              </label>
              <label class="opt-row">
                <input v-model="options.stripControlChars" type="checkbox" />
                <span>Strip invisible control characters (fixes □ tofu in pasted text)</span>
              </label>
              <label class="opt-row">
                <input v-model="options.sortKeys" type="checkbox" />
                <span>Sort keys in output</span>
              </label>
              <button class="reset-btn" @click="resetOptions">Reset to Defaults</button>
            </div>
          </div>
        </div>
        <FindBar
          v-if="showFindInput"
          ref="findBarInput"
          v-model="findInputQuery"
          :count="inputMatches.length"
          :index="inputFindIndex"
          :show-nav="true"
          :case-sensitive="caseSensitive"
          @prev="gotoInputMatch(-1)"
          @next="gotoInputMatch(1)"
          @close="closeFindInput"
          @update:case-sensitive="caseSensitive = $event"
        />
        <div class="editor-wrap">
          <div ref="gutterEl" class="gutter" aria-hidden="true"><span
            v-for="n in lineCount"
            :key="n"
            class="gutter-num"
          >{{ n }}</span></div>
          <textarea
            ref="editorEl"
            v-model="input"
            class="editor"
            spellcheck="false"
            placeholder='Paste JSON here… e.g. {"hello": "world"}'
            @keydown="onEditorKeydown"
            @paste="onEditorPaste"
            @scroll="syncGutter"
          />
        </div>
      </section>

      <!-- 拖拽分隔条 -->
      <div
        class="divider"
        role="separator"
        aria-orientation="vertical"
        aria-label="Resize panes"
        tabindex="0"
        @pointerdown="startDrag"
        @keydown="onDividerKeydown"
      />

      <!-- Output（右） -->
      <section class="pane" @pointerdown="lastPane = 'output'">
        <div class="pane-header">
          <span class="pane-label">Output:</span>
          <span class="spacer" />
          <select v-model="options.indent" class="indent-select" title="Indentation">
            <option value="twoSpaces">2 spaces</option>
            <option value="fourSpaces">4 spaces</option>
            <option value="tab">Tab</option>
            <option value="minify">Minify</option>
          </select>
          <button
            class="icon-btn"
            :disabled="!outputText"
            :title="copied ? 'Copied!' : 'Copy output'"
            @click="copyOutput()"
          >
            <Icon :name="copied ? 'check' : 'copy'" />
            {{ copied ? 'Copied' : 'Copy' }}
          </button>
        </div>
        <FindBar
          v-if="showFindOutput"
          ref="findBarOutput"
          v-model="findOutputQuery"
          :count="outputMatches"
          :index="0"
          :show-nav="false"
          :case-sensitive="caseSensitive"
          @close="closeFindOutput"
          @update:case-sensitive="caseSensitive = $event"
        />
        <pre class="output" :style="{ '--j-indent': treeIndent }"><template v-if="hasTree"><div
  v-for="(ln, i) in flatLines"
  :key="i"
  class="j-line"
  :style="{ paddingLeft: `calc(var(--j-indent) * ${ln.level})` }"
><template v-if="ln.keySegs"><span
    v-for="(s, j) in ln.keySegs"
    :key="j"
    :class="[s.cls, { 'j-mark': s.hit }]"
  >{{ s.t }}</span><span class="j-punct">: </span></template><span
    v-if="ln.toggle"
    class="j-toggle"
    role="button"
    tabindex="0"
    @click="toggleNode(ln.path)"
    @keydown.enter="toggleNode(ln.path)"
  >{{ ln.toggle.bracket }}{{ ln.toggle.collapsed ? '+' : '−' }}</span><span
    v-if="ln.preview"
    class="j-preview"
    @click="toggleNode(ln.path)"
  >{{ ln.preview }}</span><span
    v-for="(s, j) in ln.value"
    :key="j"
    :class="[s.cls, { 'j-mark': s.hit }]"
  >{{ s.t }}</span><span v-if="ln.comma" class="j-punct">,</span></div></template><template v-else-if="outputText"><span v-for="(s, i) in outputTextSegs" :key="i" :class="{ 'j-mark': s.hit }">{{ s.t }}</span></template><span v-else class="output-placeholder">Formatted JSON will appear here…</span></pre>
      </section>
    </div>

    <!-- 状态栏 -->
    <div class="statusbar" :class="status.cls">
      <span v-if="outcome" class="dot" />
      <span class="status-text">{{ clipboardError || status.text }}</span>
      <span v-if="status.cls === 'ok' && outcome?.repaired" class="status-detail">auto-repaired</span>
      <span v-if="warningsText" class="status-detail">{{ warningsText }}</span>
      <span class="stats">{{ statsText }}</span>
    </div>
  </div>
</template>
