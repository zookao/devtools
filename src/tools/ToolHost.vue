<script setup lang="ts">
// import Icon from '../components/Icon.vue'
import type { ToolDef } from './registry'

const props = defineProps<{ tool: ToolDef }>()
</script>

<template>
  <div class="tool-host">
    <header class="tool-header">
      <!-- <Icon :name="props.tool.icon" /> -->
      <h1>{{ props.tool.title }}</h1>
    </header>
    <KeepAlive>
      <component :is="props.tool.component" :title="props.tool.title" class="tool-slot" />
    </KeepAlive>
  </div>
</template>

<style scoped>
.tool-host {
  display: flex;
  flex-direction: column;
  flex: 1;
  min-height: 0;
}

.tool-slot {
  flex: 1;
  min-height: 0;
  display: flex;
  flex-direction: column;
}
</style>
