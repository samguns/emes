<script setup lang="ts">
import Classification from '../components/Classification.vue';
</script>

<template>
    <div class="view-container">
        <Classification />
    </div>
</template>

<style>
@media (min-width: 1024px) {
  .view-container {
    min-height: 100vh;
    display: flex;
    align-items: center;
  }
}
</style>
