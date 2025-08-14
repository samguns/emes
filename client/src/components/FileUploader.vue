<template>
    <div class="file-uploader">
      <q-card class="upload-card">
        <q-card-section>
          <div class="text-h6 q-mb-md">File Upload</div>
          
          <!-- File Input Area -->
          <div class="upload-area" @click="triggerFileInput" @drop="handleDrop" @dragover.prevent @dragenter.prevent>
            <input
              ref="fileInput"
              type="file"
              @change="handleFileSelect"
              style="display: none"
              multiple
            />
            
            <div class="upload-content">
              <q-icon name="cloud_upload" size="48px" color="primary" />
              <div class="text-h6 q-mt-md">Click to browse files</div>
              <div class="text-caption text-grey-6">or drag and drop files here</div>
            </div>
          </div>
  
          <!-- Selected Files Display -->
          <div v-if="selectedFiles.length > 0" class="q-mt-md">
            <div class="text-subtitle2 q-mb-sm">Selected Files:</div>
            <div class="file-list">
              <q-chip
                v-for="(file, index) in selectedFiles"
                :key="index"
                :label="file.name"
                :icon="getFileIcon(file.type)"
                removable
                @remove="removeFile(index)"
                class="q-ma-xs"
              >
                <q-tooltip>{{ formatFileSize(file.size) }}</q-tooltip>
              </q-chip>
            </div>
          </div>
  
          <!-- Upload Button -->
          <q-btn
            v-if="selectedFiles.length > 0"
            :loading="isUploading"
            :disable="isUploading"
            color="primary"
            label="Upload Files"
            icon="upload"
            class="q-mt-md full-width"
            @click="uploadFiles"
          />
  
          <!-- Upload Progress -->
          <div v-if="isUploading" class="q-mt-md">
            <q-linear-progress
              :value="uploadProgress"
              color="primary"
              class="q-mb-sm"
            />
            <div class="text-caption text-center">
              Uploading... {{ Math.round(uploadProgress * 100) }}%
            </div>
          </div>
  
          <!-- Status Messages -->
          <div v-if="uploadStatus" class="q-mt-md">
            <q-banner
              :class="uploadStatus.type === 'success' ? 'bg-positive' : 'bg-negative'"
              class="text-white"
            >
              <template v-slot:avatar>
                <q-icon
                  :name="uploadStatus.type === 'success' ? 'check_circle' : 'error'"
                  color="white"
                />
              </template>
              {{ uploadStatus.message }}
            </q-banner>
          </div>
        </q-card-section>
      </q-card>
    </div>
  </template>
  
  <script setup lang="ts">
  import { ref } from 'vue'
  import { useQuasar } from 'quasar'
import axios from 'axios'
  
  interface UploadStatus {
    type: 'success' | 'error'
    message: string
  }
  
  const $q = useQuasar()
  
  // Reactive state
  const fileInput = ref<HTMLInputElement>()
  const selectedFiles = ref<File[]>([])
  const isUploading = ref(false)
  const uploadProgress = ref(0)
  const uploadStatus = ref<UploadStatus | null>(null)
  
  // Constants
  const UPLOAD_URL = 'http://localhost:8642/api/upload'
  
  // Methods
  const triggerFileInput = () => {
    fileInput.value?.click()
  }
  
  const handleFileSelect = (event: Event) => {
    const target = event.target as HTMLInputElement
    if (target.files) {
      const newFiles = Array.from(target.files)
      selectedFiles.value = [...selectedFiles.value, ...newFiles]
      clearStatus()
    }
  }
  
  const handleDrop = (event: DragEvent) => {
    event.preventDefault()
    if (event.dataTransfer?.files) {
      const droppedFiles = Array.from(event.dataTransfer.files)
      selectedFiles.value = [...selectedFiles.value, ...droppedFiles]
      clearStatus()
    }
  }
  
  const removeFile = (index: number) => {
    selectedFiles.value.splice(index, 1)
    clearStatus()
  }
  
  const getFileIcon = (mimeType: string): string => {
    if (mimeType.startsWith('image/')) return 'image'
    if (mimeType.startsWith('video/')) return 'video_file'
    if (mimeType.startsWith('audio/')) return 'audio_file'
    if (mimeType.includes('pdf')) return 'picture_as_pdf'
    if (mimeType.includes('word') || mimeType.includes('document')) return 'description'
    if (mimeType.includes('spreadsheet') || mimeType.includes('excel')) return 'table_chart'
    if (mimeType.includes('presentation') || mimeType.includes('powerpoint')) return 'slideshow'
    if (mimeType.includes('zip') || mimeType.includes('archive')) return 'archive'
    return 'insert_drive_file'
  }
  
  const formatFileSize = (bytes: number): string => {
    if (bytes === 0) return '0 Bytes'
    const k = 1024
    const sizes = ['Bytes', 'KB', 'MB', 'GB']
    const i = Math.floor(Math.log(bytes) / Math.log(k))
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
  }
  
  const clearStatus = () => {
    uploadStatus.value = null
    uploadProgress.value = 0
  }
  
  const uploadFiles = async () => {
    if (selectedFiles.value.length === 0) return
  
    isUploading.value = true
    uploadProgress.value = 0
    clearStatus()
  
    try {
      const totalFiles = selectedFiles.value.length
      let completedFiles = 0
  
      for (const file of selectedFiles.value) {
        const formData = new FormData()
        formData.append('file', file)
  
        // A "network error" in axios usually means the request could not reach the server at all,
        // often due to CORS issues, server not running, wrong URL, or network connectivity problems.
        // It is NOT a normal HTTP error (like 404 or 500), but a failure to make the request.
        // To help debug, you can add a try/catch here and log the error details:
        let response;
        try {
          response = await axios.post(UPLOAD_URL, formData, {
            headers: {
              'Accept': 'application/json',
              'Content-Type': 'multipart/form-data'
            }
          });
        } catch (err) {
          // Axios "network error" is usually in err.message and err.request
          console.error('Axios network error:', err);
          throw err; // rethrow so the outer catch handles it
        }

        // Axios responses do not have an 'ok' property like fetch.
        // Instead, check for a 2xx status code.
        if (response.status < 200 || response.status >= 300) {
          throw new Error(`Upload failed for ${file.name}: ${response.statusText}`)
        }

        completedFiles++
        uploadProgress.value = completedFiles / totalFiles
      }
  
      // All files uploaded successfully
      uploadStatus.value = {
        type: 'success',
        message: `Successfully uploaded ${totalFiles} file(s)`
      }
  
      // Clear selected files after successful upload
      selectedFiles.value = []
      
      // Show success notification
      $q.notify({
        type: 'positive',
        message: `Successfully uploaded ${totalFiles} file(s)`,
        position: 'top'
      })
  
    } catch (error) {
      console.error('Upload error:', error)
      uploadStatus.value = {
        type: 'error',
        message: error instanceof Error ? error.message : 'Upload failed'
      }
  
      // Show error notification
      $q.notify({
        type: 'negative',
        message: 'Upload failed. Please try again.',
        position: 'top'
      })
    } finally {
      isUploading.value = false
    }
  }
  </script>
  
  <style scoped>
  .file-uploader {
    max-width: 600px;
    margin: 0 auto;
  }
  
  .upload-card {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }
  
  .upload-area {
    border: 2px dashed #ccc;
    border-radius: 8px;
    padding: 2rem;
    text-align: center;
    cursor: pointer;
    transition: all 0.3s ease;
    background-color: #fafafa;
  }
  
  .upload-area:hover {
    border-color: var(--q-primary);
    background-color: #f0f8ff;
  }
  
  .upload-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
  }
  
  .file-list {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  
  .q-chip {
    max-width: 200px;
  }
  
  .q-chip .q-chip__content {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  </style>