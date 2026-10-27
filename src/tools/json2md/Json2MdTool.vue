<script setup lang="ts">
import { computed, ref } from 'vue'
import Icon from '../../components/Icon.vue'
import { clipboardWriteText, json2mdConvert } from '../../api'

/** 示例 JSON：与 Go 版 json2md 保持一致，含 // 注释 */
const SAMPLE = `{
  "id": 12, //必填，发票id
  "invoice_date": "20260310", //必填，开票日期
  "invoice_number": "11111111111111", //必填，发票号
  "actual_amount": "40000", //非必填，实际开票金额
  "invoice_file": [
    {
      "file_name": "1-20240614 (2) (1).pdf",
      "file_url": "/pdf/pdf_1665_1772433993326.pdf",
      "file_url_format": "https://example.com/pdf/pdf_1665.pdf?Expires=6698467986"
    }
  ], //必填，附件
  "organ": {
    "organ_id": 1,
    "organ_name": "测试机构"
  } //必填，机构信息
}`

interface TableData {
  headers: string[]
  rows: string[][]
}

const input = ref('')
const markdown = ref('')
const errorMsg = ref('')
const computing = ref(false)
const copied = ref(false)

const table = computed<TableData | null>(() => parseTable(markdown.value))

/** 解析后端生成的 Markdown 表格（首行表头，第二行分隔符） */
function parseTable(md: string): TableData | null {
  const lines = md.trim().split('\n')
  if (lines.length < 2) return null

  const splitRow = (line: string) =>
    line
      .replace(/^\s*\|/, '')
      .replace(/\|\s*$/, '')
      .split('|')
      .map((c) => c.trim())

  const headers = splitRow(lines[0])
  const rows = lines.slice(2).filter((l) => l.trim()).map(splitRow)
  if (!headers.length) return null
  return { headers, rows }
}

async function convert() {
  const text = input.value.trim()
  if (!text || computing.value) return
  computing.value = true
  errorMsg.value = ''
  try {
    markdown.value = await json2mdConvert(text)
  } catch (e) {
    markdown.value = ''
    errorMsg.value = String(e)
  } finally {
    computing.value = false
  }
}

function loadSample() {
  input.value = SAMPLE
  markdown.value = ''
  errorMsg.value = ''
}

function clearAll() {
  input.value = ''
  markdown.value = ''
  errorMsg.value = ''
}

async function copyMarkdown() {
  if (!markdown.value) return
  await clipboardWriteText(markdown.value)
  copied.value = true
  setTimeout(() => (copied.value = false), 1500)
}
</script>

<template>
  <div class="tool-body">
    <div class="j2m-body">
      <div class="j2m-toolbar">
        <div class="hash-actions">
          <button class="icon-btn" @click="loadSample()">
            <Icon name="flask" />
            <span>加载示例</span>
          </button>
          <button class="icon-btn" @click="clearAll()">
            <Icon name="x" />
            <span>清空</span>
          </button>
        </div>
        <button class="hash-compute" :disabled="!input.trim() || computing" @click="convert()">
          <Icon name="swap" />
          <span>{{ computing ? '转换中…' : '转换' }}</span>
        </button>
      </div>

      <p v-if="errorMsg" class="hash-error">{{ errorMsg }}</p>

      <div class="j2m-grid">
        <section class="hash-col">
          <div class="hash-col-head">
            <span class="hash-col-label">输入 JSON</span>
          </div>
          <textarea
            v-model="input"
            class="hash-textarea"
            spellcheck="false"
            placeholder="在此输入 JSON，可用 // 注释标注必填与说明，例如：&#10;{&#10;  &quot;id&quot;: 12, //必填，发票id&#10;  &quot;remark&quot;: &quot;x&quot; //非必填，备注&#10;}"
          />
        </section>

        <section class="hash-col">
          <div class="hash-col-head">
            <span class="hash-col-label">Markdown 预览</span>
          </div>
          <div class="j2m-preview">
            <table v-if="table" class="j2m-table">
              <thead>
                <tr>
                  <th v-for="(h, i) in table.headers" :key="i">{{ h }}</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="(row, ri) in table.rows" :key="ri">
                  <td v-for="(cell, ci) in row" :key="ci">{{ cell }}</td>
                </tr>
              </tbody>
            </table>
            <p v-else class="j2m-placeholder">点击「转换」按钮查看结果</p>
          </div>
        </section>

        <section class="hash-col">
          <div class="hash-col-head">
            <span class="hash-col-label">Markdown 代码</span>
            <button
              class="ts-icon-btn"
              :title="copied ? 'Copied!' : 'Copy markdown'"
              :disabled="!markdown"
              @click="copyMarkdown()"
            >
              <Icon :name="copied ? 'check' : 'copy'" />
            </button>
          </div>
          <textarea
            class="hash-textarea readonly"
            readonly
            spellcheck="false"
            :value="markdown"
            placeholder="转换后的 Markdown 代码将显示在这里..."
          />
        </section>
      </div>
    </div>
  </div>
</template>
