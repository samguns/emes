<template>
  <q-table
    :rows="filteredRows"
    :columns="columns"
    row-key="id"
    v-model:pagination="pagination"
    :rows-per-page-options="[0]"
    :loading="loading"
    :rows-per-page-label="'Rows per page:'"
    :no-data-label="'No data available'"
    class="classification-table"
    aria-label="Classification data table"
    selection="multiple"
    v-model:selected="selected"
    @selection="onSelectionChange"
    
    virtual-scroll
    :virtual-scroll-item-size="48"
    :virtual-scroll-sticky-size-start="48"
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
      <q-btn color="primary" label="训练分类模型" icon="play_arrow" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';

// Constants
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || `http://${window.location.hostname}:8642`;
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

// Function to get the class label from the class value
function getClassLabel(classValue: number): string {
  return CLASS_LABELS[classValue] || `未知类型 (${classValue})`;
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
  { 
    name: 'select',
    required: true,
    label: ' ',
    align: 'center',
    field: 'id',
    sortable: false 
  },
  { name: 'name', required: true, label: '名字', align: 'left', field: 'name', sortable: true },
  { 
    name: 'class', 
    required: true, 
    label: '类别', 
    align: 'left', 
    field: row => getClassLabel(row.class), 
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
  rowsPerPage: 10,
  rowsNumber: 0,
  sortBy: 'created_at',
  descending: false
});

const filter = ref('');
const loading = ref(false);
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
    
    originalRows.value = data.data.entries;
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
async function onRequest(props: any) {
  const { page, rowsPerPage } = props.pagination;
  
  pagination.value.page = page;
  pagination.value.rowsPerPage = rowsPerPage;
  
  await fetchData(page, rowsPerPage);
}

async function onScroll(event: any) {
    console.log('Scroll event:', event);
  // Handle virtual scroll events
  if (event && event.target) {
    const { scrollTop, clientHeight, scrollHeight } = event.target;
    if (scrollTop + clientHeight >= scrollHeight) {
      // User has scrolled to the bottom, load more data
      pagination.value.page++;
      await fetchData(pagination.value.page, pagination.value.rowsPerPage);
    }
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
  await fetchData(pagination.value.page, pagination.value.rowsPerPage);
}

// Selection change handler
function onSelectionChange(details: { added: boolean, keys: [], rows: readonly TableRow[] }) {
  if (details.added) {
    selectedFiles.value = details.rows.map(row => row.name);
  } else {
    selectedFiles.value = selectedFiles.value.filter(name => !details.rows.map(row => row.name).includes(name));
  }
  // Update selectedFiles with the names of selected rows
  
  console.log('Selected files:', selectedFiles.value);
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
watch(filter, () => {
  pagination.value.page = 1;
});

// OnMounted
onMounted(async () => {
  await fetchData(pagination.value.page, pagination.value.rowsPerPage);
});
</script>

<style scoped>
.classification-table {
  margin: 8px;
  min-height: 400px;
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

