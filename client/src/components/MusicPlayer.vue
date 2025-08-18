<template>
  <q-layout view="hHh lpR fFf">
    <!-- <q-header elevated>
      <q-toolbar>
        <q-toolbar-title>
          FFmpeg REST Player
          <q-badge v-if="props.class" color="accent" class="q-ml-sm">
            Class: {{ props.class }}
          </q-badge>
        </q-toolbar-title>
        <div class="row items-center q-gutter-sm">
          <q-btn flat dense round icon="refresh" @click="refresh" :loading="loading" />
          <q-select
            v-model="selectedPlayer"
            :options="playerOptions"
            option-label="label"
            option-value="value"
            dense filled outlined
            style="min-width: 240px"
            label="Output player"
            @update:model-value="setPlayer"
          />
        </div>
      </q-toolbar>
    </q-header> -->

    <q-page-container>
      <q-page class="q-pa-md">
        <div class="row q-col-gutter-xl">
          <div class="col-12 col-md-8">
            <q-card flat bordered class="q-pa-md">
              <div class="text-h6">{{ currentTitle || '—' }}</div>
              <div class="text-caption text-grey-7">
                {{ status.position || '--:--' }} / {{ status.duration || '--:--' }}
                <span v-if="status.paused" class="text-negative q-ml-sm">[PAUSED]</span>
              </div>

              <div class="q-mt-md">
                <q-slider
                  v-model="scrubSeconds"
                  :min="0"
                  :max="durationSec || 0"
                  :step="0.25"
                  label
                  :label-value="formatTime(scrubSeconds)"
                  @change="onSeekTo"
                  :disable="!durationSec"
                />
                <q-linear-progress
                  :value="progress"
                  rounded
                  size="10px"
                  class="q-mt-sm"
                />
              </div>

              <div class="q-mt-md row items-center q-gutter-sm">
                <q-btn color="grey-8" icon="skip_previous" round @click="prev" />
                <q-btn :color="status.paused ? 'primary' : 'primary'" :icon="status.paused ? 'play_arrow' : 'pause'" round @click="toggle" />
                <q-btn color="grey-8" icon="skip_next" round @click="next" />
                <q-separator vertical class="q-mx-md" />
                <q-btn color="grey-8" dense outline label="-5s" @click="seek(-5)" />
                <q-btn color="grey-8" dense outline label="+5s" @click="seek(5)" />
                <q-separator vertical class="q-mx-md" />
                <div class="row items-center">
                  <q-icon name="volume_up" class="q-mr-sm" />
                  <q-slider
                    v-model="volumePercent"
                    :min="0"
                    :max="400"
                    :step="1"
                    style="min-width: 220px"
                    @change="setVolumePercent"
                  />
                  <div class="q-ml-sm" style="width: 3rem; text-align:right">{{ volumePercent }}%</div>
                </div>
              </div>
            </q-card>

            <q-card flat bordered class="q-pa-md q-mt-lg">
              <div class="row items-center q-gutter-sm">
                <q-input v-model="playIndexInput" type="number" label="Play index" filled dense style="width: 160px" />
                <q-btn label="Play" color="primary" @click="playIndex" />
                <q-btn label="Stop" color="negative" outline @click="stop" />
              </div>
              <div class="text-caption text-grey q-mt-sm">
                Tip: indices are 0-based. Adding files requires server-side paths via <code>/load</code> or <code>/enqueue</code>.
              </div>
            </q-card>
          </div>

          <div class="col-12 col-md-4">
            <q-card flat bordered class="q-pa-md">
              <div class="text-subtitle1 q-mb-sm">Playlist</div>
              <q-list bordered separator>
                <q-item
                  v-for="(p, i) in status.playlist"
                  :key="p + i"
                  clickable
                  :active="status.index === i"
                  active-class="bg-primary text-white"
                  @click="play(i)"
                >
                  <q-item-section>
                    <div class="ellipsis">{{ basename(p) }}</div>
                    <div class="text-caption text-grey-7 ellipsis">{{ p }}</div>
                  </q-item-section>
                  <q-item-section side>
                    <q-badge v-if="status.index === i" color="primary" label="Now" />
                  </q-item-section>
                </q-item>
                <q-item v-if="!status.playlist?.length">
                  <q-item-section>Playlist empty</q-item-section>
                </q-item>
              </q-list>
            </q-card>

            <q-card flat bordered class="q-pa-md q-mt-lg">
              <div class="text-subtitle1 q-mb-sm">Raw status</div>
              <q-scroll-area style="height: 240px">
                <pre class="text-caption">{{ status }}</pre>
              </q-scroll-area>
            </q-card>
          </div>
        </div>
      </q-page>
    </q-page-container>
  </q-layout>
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount, reactive, ref, computed } from 'vue';
import { api } from '@/services/axios';

// Define props for the component
const props = defineProps({
  class: { type: String, default: '' }
});

type PlayerOpt = { label: string; value: number | null };

const status = reactive<any>({
  playlist: [],
  index: null,
  track: null,
  position_sec: 0,
  position: '--:--',
  duration_sec: 0,
  duration: '--:--',
  paused: false,
  eof: false,
  volume: 1.0,
  player: null
});

const loading = ref(false);
const pollTimer = ref<number | null>(null);

const scrubSeconds = ref(0);
const playIndexInput = ref<number>(0);

const durationSec = computed(() => Number(status.duration_sec || 0));
const progress = computed(() => (durationSec.value > 0 ? (Number(status.position_sec || 0) / durationSec.value) : 0));
const currentTitle = computed(() => (status.track ? basename(status.track) : null));

const volumePercent = ref(100);
const playerOptions = ref<PlayerOpt[]>([{ label: 'Default player', value: null }]);
const selectedPlayer = ref<number | null>(null);

function basename(p: string) {
  return (p || '').split(/[\\/]/).pop() || p;
}

function formatTime(s: number) {
  if (!isFinite(s) || s < 0) return '--:--';
  const sec = Math.floor(s);
  const mm = Math.floor(sec / 60);
  const ss = sec % 60;
  const hh = Math.floor(mm / 60);
  const m2 = mm % 60;
  return hh ? `${hh}:${String(m2).padStart(2,'0')}:${String(ss).padStart(2,'0')}` : `${String(mm).padStart(2,'0')}:${String(ss).padStart(2,'0')}`;
}

async function refresh() {
  loading.value = true;
  try {
    // Use the class parameter if provided
    const endpoint = props.class ? `/api/player/status?class=${encodeURIComponent(props.class)}` : '/status';
    const { data } = await api.get(endpoint);
    Object.assign(status, data);
    // sync sliders
    scrubSeconds.value = Number(status.position_sec || 0);
    volumePercent.value = Math.round(Number(status.volume || 1) * 100);
    selectedPlayer.value = status.player ?? null;
  } finally {
    loading.value = false;
  }
}

async function toggle() { 
  await api.post('/toggle', props.class ? { class: props.class } : {}); 
  await refresh(); 
}

async function next() { 
  await api.post('/next', props.class ? { class: props.class } : {}); 
  await refresh(); 
}

async function prev() { 
  await api.post('/prev', props.class ? { class: props.class } : {}); 
  await refresh(); 
}

async function stop() { 
  await api.post('/stop', props.class ? { class: props.class } : {}); 
  await refresh(); 
}

async function play(index?: number) {
  const payload: { index?: number; class?: string } = index != null ? { index } : {};
  if (props.class) {
    payload.class = props.class;
  }
  await api.post('/play', payload);
  await refresh();
}

async function playIndex() {
  if (playIndexInput.value == null) return;
  await play(Number(playIndexInput.value));
}

async function seek(delta: number) {
  const payload: { delta: number; class?: string } = { delta };
  if (props.class) {
    payload.class = props.class;
  }
  await api.post('/seek', payload);
  await refresh();
}

async function onSeekTo(v: number | undefined = undefined) {
  const seconds = (typeof v === 'number' ? v : scrubSeconds.value) || 0;
  const payload: { seconds: number; class?: string } = { seconds };
  if (props.class) {
    payload.class = props.class;
  }
  await api.post('/seek_to', payload);
  await refresh();
}

async function setVolumePercent() {
  const value = Math.max(0, Math.min(400, volumePercent.value)) / 100.0; // -> 0..4
  const payload: { value: number; class?: string } = { value };
  if (props.class) {
    payload.class = props.class;
  }
  await api.post('/volume', payload);
  await refresh();
}

async function listPlayer() {
  // Use the class parameter if provided
  const endpoint = props.class ? `/api/player?class=${encodeURIComponent(props.class)}` : '/devices';
  const { data } = await api.get(endpoint);
//   playerOptions.value = [{ label: 'Default player', value: null }].concat(
//     data.map((d: any) => ({
//       label: `[${d.index}] ${d.name} — ${d.max_output_channels}ch`,
//       value: d.index
//     }))
//   );
}

async function setPlayer() {
  const payload: { id: number | null; class?: string } = { id: selectedPlayer.value };
  if (props.class) {
    payload.class = props.class;
  }
  await api.post('/api/player', payload);
  await refresh();
}

onMounted(async () => {
  await refresh();
  await listPlayer();
  pollTimer.value = window.setInterval(refresh, 1000);
});

onBeforeUnmount(() => {
  if (pollTimer.value) window.clearInterval(pollTimer.value);
});
</script>

<style>
pre { white-space: pre-wrap; }
</style>
