<script setup lang="ts">
import Icon from '../components/Icon.vue'

defineProps<{ title: string }>()
</script>

<template>
  <div class="placeholder">
    <Icon name="tool" />
    <h2>{{ title }}</h2>
    <p>Coming soon…</p>
  </div>
</template>
