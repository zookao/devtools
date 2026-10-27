<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import Icon from '../../components/Icon.vue'
import { clipboardWriteText, unicodeDecode, unicodeEncode } from '../../api'

type Mode = 'encode' | 'decode'

const ENCODE_SAMPLE = '你好，世界！'
const DECODE_SAMPLE = '\\u4f60\\u597d\\uff0c\\u4e16\\u754c\\uff01'

const mode = ref<Mode>('encode')
const input = ref('')
const result = ref('')
const copied = ref(false)

const resultLabel = computed(() => (mode.value === 'encode' ? '编码结果' : '解码结果'))

/** 输入变化后自动转换（无需点按钮） */
let seq = 0
let timer: number | undefined

async function convert() {
  const mySeq = ++seq
  const text = input.value
  if (text === '') {
    result.value = ''
    return
  }
  const out = mode.value === 'encode' ? await unicodeEncode(text) : await unicodeDecode(text)
  // 丢弃过期响应，避免快速输入时结果错乱
  if (mySeq === seq) result.value = out
}

watch([input, mode], () => {
  window.clearTimeout(timer)
  timer = window.setTimeout(convert, 120)
})

function loadSample() {
  input.value = mode.value === 'encode' ? ENCODE_SAMPLE : DECODE_SAMPLE
}

function clearAll() {
  input.value = ''
  result.value = ''
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
    <div class="hash-body">
      <div class="b64-toolbar">
        <div class="uni-left">
          <div class="b64-tabs" role="tablist">
            <button
              class="b64-tab"
              :class="{ active: mode === 'encode' }"
              role="tab"
              :aria-selected="mode === 'encode'"
              @click="mode = 'encode'"
            >
              编码
            </button>
            <button
              class="b64-tab"
              :class="{ active: mode === 'decode' }"
              role="tab"
              :aria-selected="mode === 'decode'"
              @click="mode = 'decode'"
            >
              解码
            </button>
          </div>
          <span class="uni-hint">将文本转换为Unicode编码或将Unicode编码转换为文本</span>
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
            <span class="hash-col-label">{{ mode === 'encode' ? '输入文本' : '输入编码' }}</span>
          </div>
          <textarea
            v-model="input"
            class="hash-textarea"
            spellcheck="false"
            :placeholder="mode === 'encode' ? '请输入需要编码的文本...' : '请输入需要解码的 Unicode 编码...'"
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

      <div class="uni-copy-row">
        <button class="icon-btn" :disabled="!result" @click="copyResult()">
          <Icon :name="copied ? 'check' : 'copy'" />
          <span>{{ copied ? '已复制' : '复制结果' }}</span>
        </button>
      </div>

      <section class="hash-note">
        <h2>编码说明</h2>
        <p>
          Unicode编码使用u前跟最后四位十六进制数字表示字符，可以表示几乎所有语言的字符，如中文、日文等。
        </p>
      </section>
    </div>
  </div>
</template>
