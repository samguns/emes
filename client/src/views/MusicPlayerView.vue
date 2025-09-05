<script setup lang="ts">
import MusicPlayer from '@/components/MusicPlayer.vue'

// Define props to receive route parameters
const props = defineProps({
  class: { type: String, default: '' }
});
</script>

<template>
    <div class="music-player-view">
        <MusicPlayer :class="props.class" />
    </div>
</template>

<style scoped>
.music-player-view {
    padding: 0;
    max-width: 900px;
    height: 90vh;
    margin: 0 auto;
    overflow-y: auto;
}
</style>