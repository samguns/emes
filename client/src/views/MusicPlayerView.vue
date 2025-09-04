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
    padding: 20px;
    max-width: 800px;
    margin: 0 auto;
}
</style>