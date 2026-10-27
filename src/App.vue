<script setup lang="ts">
import { computed, ref } from 'vue'
import Sidebar from './components/Sidebar.vue'
import ToolHost from './tools/ToolHost.vue'
import { TOOLS } from './tools/registry'

const ACTIVE_KEY = 'devtools:active-tool'

const activeId = ref(localStorage.getItem(ACTIVE_KEY) || 'json')
if (!TOOLS.some((t) => t.id === activeId.value)) activeId.value = 'json'

const activeTool = computed(() => TOOLS.find((t) => t.id === activeId.value) ?? TOOLS[0])

function onSelect(id: string) {
  activeId.value = id
  localStorage.setItem(ACTIVE_KEY, id)
}
</script>

<template>
  <div class="app">
    <Sidebar :active-id="activeId" @select="onSelect" />
    <main class="content">
      <ToolHost :tool="activeTool" />
    </main>
  </div>
</template>
