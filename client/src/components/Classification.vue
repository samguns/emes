<template>
  <q-table
    :rows="filteredRows"
    :columns="columns"
    row-key="id"
    v-model:pagination="pagination"
    :rows-per-page-options="[0]"
    :loading="loading"

    :no-data-label="'No data available'"
    class="classification-table"
    aria-label="Classification data table"
    selection="multiple"
    v-model:selected="selected"
    @selection="onSelectionChange"
    
    virtual-scroll
    :virtual-scroll-sticky-size-start="18"
    @virtual-scroll="onScroll"
  >
    <template v-slot:top>
      <div class="row full-width items-center q-gutter-md">
        <q-input
          outlined
          v-model="filter"
          debounce="300"
          placeholder="搜索..."
          class="col-grow"
          @update:model-value="onFilter"
          clearable
          aria-label="Search table content"
        >
          <template v-slot:prepend>
            <q-icon name="search" />
          </template>
        </q-input>
        <q-btn
          v-if="filter"
          @click="clearFilter"
          color="secondary"
          icon="clear"
          label="Clear"
          aria-label="Clear search filter"
        />
        <q-btn
          @click="refreshData"
          color="primary"
          icon="refresh"
          label="刷新"
          aria-label="Refresh data"
          :loading="loading"
        />
      </div>
    </template>
    
    <template v-slot:no-data>
      <div class="full-width row flex-center q-pa-lg text-grey-6">
        <q-icon name="search_off" size="2rem" class="q-mr-sm" />
        <span v-if="error">{{ error }}</span>
        <span v-else>No items found matching your search criteria</span>
      </div>
    </template>

    <template v-slot:loading>
      <q-inner-loading showing color="primary" />
    </template>
    
    <!-- Custom template for name column with tooltip -->
    <template v-slot:body-cell-name="props">
      <q-td :props="props">
        <span>
          {{ props.row.name.length > 15 ? props.row.name.substring(0, 15) + '...' : props.row.name }}
          <q-tooltip>
            {{ getTooltip(props.row) }}
          </q-tooltip>
        </span>
      </q-td>
    </template>
    
    <!-- Custom template for class column with select dropdown -->
    <template v-slot:body-cell-class="props">
      <q-td :props="props">
        <q-select
          :model-value="CLASS_LABELS[props.row.class]"
          :options="classOptions"
          option-value="value"
          option-label="label"
          dense
          borderless
          :loading="updating.has(props.row.id)"
          :disable="updating.has(props.row.id)"
          @update:model-value="(newValue) => onClassChange(props.row, newValue)"
          style="min-width: 120px"
        />
      </q-td>
    </template>
  </q-table>
  
  <!-- Display selected files -->
  <div v-if="selectedFiles.length > 0" class="selected-files-panel q-mt-md">
    <!-- <div class="text-subtitle1 q-mb-sm">已选择的文件:</div>
    <q-chip
      v-for="(fileName, index) in selectedFiles" 
      :key="index"
      color="primary"
      text-color="white"
      removable
      @remove="removeSelectedFile(index)"
    >
      {{ fileName }}
    </q-chip> -->
    
    <div class="q-mt-md">
      <q-btn 
        color="primary" 
        :label="`训练分类模型 ${selectedFiles.length > 0 ? `(已选择 ${selectedFiles.length} 个文件)` : ''}`"
        icon="play_arrow"
        :disable="selectedFiles.length === 0"
      />
      <div v-if="selectedFiles.length === 0" class="text-caption text-grey q-mt-sm">
        请先选择要用于训练的文件
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import { useQuasar } from 'quasar';

// Constants
const API_BASE_URL = import.meta.env.VITE_API_BASE || 'http://' + window.location.hostname + ':8642';
const FILE_LIST_URL = `${API_BASE_URL}/api/filelist`;

const selectedFiles = ref<string[]>([]);
const selected = ref<TableRow[]>([]);

// Class name mapping
const CLASS_LABELS: Record<number, string> = {
  0: '助眠音乐',
  1: '活力音乐',
  2: '运动音乐',
  3: '其他'
};

// Create options array for class select dropdown
const classOptions = Object.entries(CLASS_LABELS).map(([value, label]) => ({
  label,
  value: parseInt(value)
}));

// Function to get the class label from the class value
function getClassLabel(classValue: number): string {
  return CLASS_LABELS[classValue] || `未知类型 (${classValue})`;
}

function getTooltip(row: TableRow): string {
  return row.name;
}

const $q = useQuasar();
const updating = ref<Set<number>>(new Set());

// Handle class change for a row
async function onClassChange(row: TableRow, newClassValue: any) {
  const oldValue = row.class;
  
  // Prevent multiple simultaneous updates for the same row
  if (updating.value.has(row.id)) {
    row.class = oldValue;
    return;
  }
  
  updating.value.add(row.id);
  
  try {
    // Show loading notification
    const loadingNotify = $q.notify({
      type: 'ongoing',
      message: `正在更新 "${row.name.length > 20 ? row.name.substring(0, 20) + '...' : row.name}" 的类别...`,
      spinner: true,
      timeout: 0
    });
    
    // Update the class in the backend
    const response = await fetch(`${API_BASE_URL}/api/filelist/update-class`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        id: row.id,
        class: newClassValue.value,
      }),
    });

    // Dismiss loading notification
    loadingNotify();

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const result = await response.json();
    console.log('update_class result: ', result);
    
    if (result.code === 0) {
      // Update successful - show success notification
      $q.notify({
        type: 'positive',
        message: `成功更新类别：${getClassLabel(oldValue)} → ${getClassLabel(newClassValue.value)}`,
        icon: 'check_circle',
        timeout: 2000
      });

      row.class = newClassValue.value;
      
      // console.log(`Successfully updated class for "${row.name}" from ${getClassLabel(oldValue)} to ${getClassLabel(newClassValue)}`);
    } else {
      throw new Error(result.message || 'Failed to update class');
    }
    
  } catch (error) {
    console.error('Error updating class:', error);
    
    // Revert the change on error
    row.class = oldValue;
    
    // Show error notification
    $q.notify({
      type: 'negative',
      message: `更新失败：${error instanceof Error ? error.message : '未知错误'}`,
      icon: 'error',
      timeout: 5000
    });
  } finally {
    updating.value.delete(row.id);
  }
}

// TypeScript interfaces
interface TableRow {
  id: number;
  class: number;
  name: string;
  is_training_data: boolean;
  created_at: number;
}

interface TableColumn {
  name: string;
  required: boolean;
  label: string;
  align: 'left' | 'right' | 'center';
  field: ((row: TableRow) => string) | keyof TableRow;
  sortable: boolean;
}

interface ApiResponse {
  code: number;
  data: {
    entries: TableRow[];
    total_entries: number;
  };
  message: string;
}

interface PaginationRequest {
  page: number;
  page_size: number;
}

// Reactive data
const originalRows = ref<TableRow[]>([]);
const columns = ref<TableColumn[]>([
  // { 
  //   name: 'select',
  //   required: true,
  //   label: ' ',
  //   align: 'center',
  //   field: 'id',
  //   sortable: false 
  // },
  {
    name: 'name',
    required: true,
    label: '名字',
    align: 'left',
    field: 'name',
    sortable: true
  },
  { 
    name: 'class', 
    required: true, 
    label: '类别', 
    align: 'left', 
    field: 'class',
    sortable: true 
  },
  { 
    name: 'is_training_data', 
    required: true, 
    label: '已经参与训练', 
    align: 'center', 
    field: row => row.is_training_data ? '是' : '否',
    sortable: true 
  },
  { 
    name: 'created_at', 
    required: true, 
    label: '上传日期', 
    align: 'left', 
    field: row => new Date(row.created_at).toLocaleString(),
    sortable: true 
  },
]);

const pagination = ref({
  page: 1,
  rowsPerPage: 0,
  rowsNumber: 0,
  sortBy: 'created_at',
  descending: false
});

const filter = ref('');
const loading = ref(false);
const nextPage = ref(1);
const error = ref<string | null>(null);

// Computed property for filtered rows
const filteredRows = computed(() => {
  if (!filter.value.trim()) {
    return originalRows.value;
  }
  
  const searchTerm = filter.value.toLowerCase().trim();
  return originalRows.value.filter(row =>
    Object.values(row).some(value =>
      String(value).toLowerCase().includes(searchTerm)
    )
  );
});

// Fetch data from API
async function fetchData(page: number, pageSize: number) {
  loading.value = true;
  error.value = null;
  
  try {
    const response = await fetch(FILE_LIST_URL, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        page: page - 1, // Convert to 0-based indexing
        page_size: pageSize,
      } as PaginationRequest),
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const data: ApiResponse = await response.json();
    
    // Append new entries to existing rows instead of replacing them
    if (page === 1) {
      // Reset rows if it's the first page
      originalRows.value = data.data.entries;
    } else {
      // Append entries for subsequent pages
      originalRows.value = [...originalRows.value, ...data.data.entries];
    }
    pagination.value.rowsNumber = data.data.total_entries;
    
  } catch (err) {
    console.error('Error fetching file list:', err);
    error.value = err instanceof Error ? err.message : 'Failed to fetch data';
    originalRows.value = [];
  } finally {
    loading.value = false;
  }
}

// Handle pagination requests
// async function onRequest(props: any) {
//   const { page, rowsPerPage } = props.pagination;
  
//   pagination.value.page = page;
//   pagination.value.rowsPerPage = rowsPerPage;
  
//   await fetchData(page, rowsPerPage);
async function onScroll({ to, ref }: { to: number; ref: any }) {
  const lastIndex = filteredRows.value.length - 1;
  const lastPage = Math.ceil(pagination.value.rowsNumber / 20);
  // console.log('onScroll', to, lastIndex, nextPage.value, lastPage);
  // Handle virtual scroll events
  if (loading.value !== true && nextPage.value < 85 && to === lastIndex) {
    // console.log('Loading more data for virtual scroll:', { nextPage: nextPage.value });
    loading.value = true;
    
    // User has scrolled to the bottom, load more data
    
    await fetchData(pagination.value.page, 20);
    pagination.value.page++;
    nextPage.value++;
    // ref.refresh();
  }
}

// Filter function
async function onFilter() {
  // Reset pagination when filtering
  pagination.value.page = 1;
  
  // For client-side filtering, we don't need to fetch from server
  // The computed property handles the filtering
}

// Clear filter function
function clearFilter() {
  filter.value = '';
  pagination.value.page = 1;
}

// Refresh data function
async function refreshData() {
  await fetchData(pagination.value.page, 20);
}

// Selection change handler
function onSelectionChange(details: {
  added: any; rows: readonly TableRow[] 
}) {
  if (details?.added) {
    selectedFiles.value.push(...details.rows.map(row => row.name));
  } else {
    selectedFiles.value = selectedFiles.value.filter(name => !details.rows.map(row => row.name).includes(name));
  }
  // Update selectedFiles with the names of selected rows
  selected.value = originalRows.value.filter(row => selectedFiles.value.includes(row.name));
  
  // console.log('Selected files:', selectedFiles.value);
}

// Remove a file from the selected files list
function removeSelectedFile(index: number) {
  // Remove from selectedFiles array
  selectedFiles.value.splice(index, 1);
  
  // Update the selected rows to match
  const remainingFiles = new Set(selectedFiles.value);
  selected.value = selected.value.filter(row => remainingFiles.has(row.name));
}

// Watch for filter changes to reset pagination
// watch(filter, () => {
//   pagination.value.page = 1;
// });

// OnMounted
onMounted(async () => {
  await fetchData(pagination.value.page, 20);
});
</script>

<style scoped>
.classification-table {
  /* margin: 8px; */
  height: 700px;

  thead tr th {
    position: sticky;
    top: 0;
    z-index: 1;
    background: white;
  }
}

/* Responsive adjustments */
@media (max-width: 768px) {
  .classification-table {
    font-size: 0.9rem;
  }
  
  .q-table__top {
    flex-direction: column;
    align-items: stretch;
  }
}

/* Custom column styling */
.q-table th:nth-child(3) { /* Training Data column */
  text-align: center;
}

.q-table td:nth-child(3) {
  text-align: center;
}

.selected-files-panel {
  padding: 16px;
  background-color: rgba(0, 0, 0, 0.03);
  border-radius: 8px;
}

.q-chip {
  margin: 4px;
}
</style>

