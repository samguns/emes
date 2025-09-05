<script setup lang="ts">
import playMusic from '../components/playMusic.vue';
</script>

<template>
    <div class="view-container">
        <playMusic />
    </div>
</template>

<style>
@media (min-width: 1024px) {
  .view-container {
    height: 90vh;
    display: flex;
    align-items: center;
    overflow-y: auto;
  }
}
</style>
