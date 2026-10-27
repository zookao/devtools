<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import Icon from '../../components/Icon.vue'
import { clipboardWriteText, jsonToYaml, yamlToJson } from '../../api'

type Mode = 'json2yaml' | 'yaml2json'

const JSON_SAMPLE = `{
  "name": "devtools",
  "version": "0.1.0",
  "server": {
    "host": "127.0.0.1",
    "ports": [80, 443],
    "tls": true
  },
  "tags": ["json", "yaml"]
}`

const YAML_SAMPLE = `name: devtools
version: 0.1.0
server:
  host: 127.0.0.1
  ports:
    - 80
    - 443
  tls: true
tags:
  - json
  - yaml
`

const mode = ref<Mode>('json2yaml')
const input = ref('')
const result = ref('')
const computing = ref(false)
const copied = ref(false)
const errorMsg = ref('')

const canCompute = computed(() => input.value.trim() !== '')
const inputLabel = computed(() => (mode.value === 'json2yaml' ? 'JSON 输入' : 'YAML 输入'))
const resultLabel = computed(() => (mode.value === 'json2yaml' ? 'YAML 输出' : 'JSON 输出'))
const placeholder = computed(() =>
  mode.value === 'json2yaml' ? '请输入需要转换的 JSON...' : '请输入需要转换的 YAML...',
)

/** 切换方向时清空结果与错误，避免残留上一次的结果 */
watch(mode, () => {
  result.value = ''
  errorMsg.value = ''
})

async function compute() {
  if (!canCompute.value || computing.value) return
  computing.value = true
  errorMsg.value = ''
  try {
    result.value =
      mode.value === 'json2yaml'
        ? await jsonToYaml(input.value)
        : await yamlToJson(input.value)
  } catch (e) {
    result.value = ''
    errorMsg.value = String(e)
  } finally {
    computing.value = false
  }
}

function loadSample() {
  input.value = mode.value === 'json2yaml' ? JSON_SAMPLE : YAML_SAMPLE
  result.value = ''
  errorMsg.value = ''
}

function clearAll() {
  input.value = ''
  result.value = ''
  errorMsg.value = ''
}

async function copyResult() {
  if (!result.value) return
  await clipboardWriteText(result.value)
  copied.value = true
  setTimeout(() => (copied.value = false), 1500)
}
</script>

<template>
  <div class="tool-body">
    <div class="hash-body yaml-body">
      <p class="hash-desc">JSON 与 YAML 数据互转，支持对象、数组、嵌套结构与中文内容。</p>

      <div class="b64-toolbar">
        <div class="b64-tabs" role="tablist">
          <button
            class="b64-tab"
            :class="{ active: mode === 'json2yaml' }"
            role="tab"
            :aria-selected="mode === 'json2yaml'"
            @click="mode = 'json2yaml'"
          >
            JSON → YAML
          </button>
          <button
            class="b64-tab"
            :class="{ active: mode === 'yaml2json' }"
            role="tab"
            :aria-selected="mode === 'yaml2json'"
            @click="mode = 'yaml2json'"
          >
            YAML → JSON
          </button>
        </div>
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
      </div>

      <div class="hash-split">
        <section class="hash-col">
          <div class="hash-col-head">
            <span class="hash-col-label">{{ inputLabel }}</span>
          </div>
          <textarea
            v-model="input"
            class="hash-textarea"
            spellcheck="false"
            :placeholder="placeholder"
          />
        </section>

        <section class="hash-col">
          <div class="hash-col-head">
            <span class="hash-col-label">{{ resultLabel }}</span>
            <button
              class="ts-icon-btn"
              :title="copied ? 'Copied!' : 'Copy result'"
              :disabled="!result"
              @click="copyResult()"
            >
              <Icon :name="copied ? 'check' : 'copy'" />
            </button>
          </div>
          <textarea
            class="hash-textarea readonly"
            readonly
            spellcheck="false"
            :value="result"
            placeholder="结果将显示在这里..."
          />
        </section>
      </div>

      <div>
        <button class="hash-compute" :disabled="!canCompute || computing" @click="compute()">
          <Icon name="swap" />
          <span>{{ computing ? '转换中…' : '转换' }}</span>
        </button>
      </div>

      <p v-if="errorMsg" class="hash-error">{{ errorMsg }}</p>

      <section class="hash-note">
        <h2>说明</h2>
        <p>JSON → YAML：保持字段原始顺序，字符串按需自动加引号。</p>
        <p>YAML → JSON：输出 2 空格缩进，数字与布尔值保留原生类型，注释不会保留。</p>
      </section>
    </div>
  </div>
</template>
