<script setup lang="ts">
import FileUploader from '../components/FileUploader.vue';
</script>

<template>
    <div class="view-container">
        <FileUploader />
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
