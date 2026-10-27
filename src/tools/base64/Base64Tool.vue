<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import Icon from '../../components/Icon.vue'
import { base64Decode, base64Encode, clipboardWriteText } from '../../api'

type Mode = 'encode' | 'decode'

const ENCODE_SAMPLE = 'Hello, World!'
const DECODE_SAMPLE = 'SGVsbG8sIFdvcmxkIQ=='

const mode = ref<Mode>('encode')
const input = ref('')
const result = ref('')
const computing = ref(false)
const copied = ref(false)
const errorMsg = ref('')

const canCompute = computed(() => input.value.trim() !== '')
const resultLabel = computed(() => (mode.value === 'encode' ? '加密结果' : '解密结果'))

/** 切换模式时清空结果与错误，避免误解码内容残留 */
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
      mode.value === 'encode'
        ? await base64Encode(input.value)
        : await base64Decode(input.value)
  } catch (e) {
    result.value = ''
    errorMsg.value = String(e)
  } finally {
    computing.value = false
  }
}

function loadSample() {
  input.value = mode.value === 'encode' ? ENCODE_SAMPLE : DECODE_SAMPLE
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
    <div class="hash-body">
      <p class="hash-desc">一种编码方式，可用于在不兼容的系统之间传输二进制数据。可编码解码。</p>

      <div class="b64-toolbar">
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
            <span class="hash-col-label">原始文本</span>
          </div>
          <textarea
            v-model="input"
            class="hash-textarea"
            spellcheck="false"
            :placeholder="mode === 'encode' ? '请输入需要编码的文本...' : '请输入需要解码的 Base64 文本...'"
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
          <Icon name="lock" />
          <span>{{ computing ? '处理中…' : mode === 'encode' ? '编码' : '解码' }}</span>
        </button>
      </div>

      <p v-if="errorMsg" class="hash-error">{{ errorMsg }}</p>

      <section class="hash-note">
        <h2>算法说明</h2>
        <p>Base64: 一种编码方式，可用于在不兼容的系统之间传输二进制数据。可编码解码。</p>
        <p>Base64不是加密算法，而是一种编码方式，常用于在文本协议中传输二进制数据。</p>
      </section>
    </div>
  </div>
</template>
