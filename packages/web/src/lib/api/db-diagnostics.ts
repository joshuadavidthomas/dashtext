/**
 * IndexedDB Diagnostic Utilities
 * 
 * Use these functions in the browser console to diagnose database issues:
 * 
 * ```js
 * import { dbDiagnostics } from '$lib/api/db-diagnostics';
 * 
 * // Check database health
 * await dbDiagnostics.checkHealth();
 * 
 * // Force database recreation (WARNING: deletes all data)
 * await dbDiagnostics.recreateDatabase();
 * 
 * // List all databases
 * await dbDiagnostics.listDatabases();
 * ```
 */

import { deleteDB } from 'idb';

export const dbDiagnostics = {
  /**
   * Checks the health of the IndexedDB database
   */
  async checkHealth(): Promise<{
    status: 'healthy' | 'unhealthy' | 'missing';
    details: Record<string, unknown>;
  }> {
    const DB_NAME = 'dashtext';
    const DB_VERSION = 2;
    
    try {
      // Try to open the database
      const request = indexedDB.open(DB_NAME, DB_VERSION);
      
      return new Promise((resolve) => {
        request.onerror = () => {
          console.error('[Diagnostics] Failed to open database:', request.error);
          resolve({
            status: 'unhealthy',
            details: {
              error: request.error?.message || 'Unknown error',
              errorName: request.error?.name
            }
          });
        };
        
        request.onsuccess = () => {
          const db = request.result;
          
          try {
            // Check object stores
            const hasObjectStore = db.objectStoreNames.contains('drafts');
            
            if (!hasObjectStore) {
              db.close();
              resolve({
                status: 'unhealthy',
                details: {
                  error: 'Object store "drafts" is missing',
                  objectStores: Array.from(db.objectStoreNames)
                }
              });
              return;
            }
            
            // Check indexes
            const tx = db.transaction('drafts', 'readonly');
            const store = tx.objectStore('drafts');
            const hasUuidIndex = store.indexNames.contains('by-uuid');
            const hasModifiedIndex = store.indexNames.contains('by-modified');
            
            tx.oncomplete = () => {
              db.close();
              
              if (!hasUuidIndex || !hasModifiedIndex) {
                resolve({
                  status: 'unhealthy',
                  details: {
                    error: 'Required indexes are missing',
                    indexes: Array.from(store.indexNames),
                    missingIndexes: [
                      !hasUuidIndex && 'by-uuid',
                      !hasModifiedIndex && 'by-modified'
                    ].filter(Boolean)
                  }
                });
              } else {
                resolve({
                  status: 'healthy',
                  details: {
                    version: db.version,
                    objectStores: Array.from(db.objectStoreNames),
                    indexes: Array.from(store.indexNames)
                  }
                });
              }
            };
            
            tx.onerror = () => {
              db.close();
              resolve({
                status: 'unhealthy',
                details: {
                  error: 'Failed to access object store',
                  transactionError: tx.error?.message
                }
              });
            };
          } catch (error) {
            db.close();
            resolve({
              status: 'unhealthy',
              details: {
                error: error instanceof Error ? error.message : String(error)
              }
            });
          }
        };
        
        request.onupgradeneeded = () => {
          // Database doesn't exist or needs upgrade
          request.transaction?.abort();
          resolve({
            status: 'missing',
            details: {
              message: 'Database does not exist or needs upgrade'
            }
          });
        };
      });
    } catch (error) {
      return {
        status: 'unhealthy',
        details: {
          error: error instanceof Error ? error.message : String(error)
        }
      };
    }
  },
  
  /**
   * Lists all IndexedDB databases in the current origin
   */
  async listDatabases(): Promise<IDBDatabaseInfo[]> {
    if (!indexedDB.databases) {
      console.warn('indexedDB.databases() is not supported in this browser');
      return [];
    }
    
    return await indexedDB.databases();
  },
  
  /**
   * Deletes the dashtext database
   * WARNING: This will delete all stored drafts!
   */
  async deleteDatabase(): Promise<void> {
    const DB_NAME = 'dashtext';
    console.warn('[Diagnostics] Deleting database. All data will be lost!');
    
    await deleteDB(DB_NAME, {
      blocked() {
        console.warn('[Diagnostics] Delete blocked - close all tabs with the app open');
      }
    });
    
    console.log('[Diagnostics] Database deleted successfully');
  },
  
  /**
   * Full diagnostic report
   */
  async fullReport(): Promise<void> {
    console.group('[IndexedDB Diagnostics Report]');
    
    console.log('1. Database Health Check:');
    const health = await this.checkHealth();
    console.log(health);
    
    console.log('\n2. All Databases:');
    const databases = await this.listDatabases();
    console.log(databases);
    
    console.log('\n3. Storage Estimate:');
    if (navigator.storage && navigator.storage.estimate) {
      const estimate = await navigator.storage.estimate();
      console.log({
        usage: estimate.usage,
        quota: estimate.quota,
        percentUsed: estimate.quota ? ((estimate.usage || 0) / estimate.quota * 100).toFixed(2) + '%' : 'N/A'
      });
    } else {
      console.log('Storage API not available');
    }
    
    console.log('\n4. Browser Info:');
    console.log({
      userAgent: navigator.userAgent,
      cookieEnabled: navigator.cookieEnabled,
      language: navigator.language
    });
    
    console.groupEnd();
  },
  
  /**
   * Exports all drafts as JSON (backup before recreation)
   */
  async exportDrafts(): Promise<unknown[]> {
    const DB_NAME = 'dashtext';
    const DB_VERSION = 2;
    
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(DB_NAME, DB_VERSION);
      
      request.onerror = () => reject(request.error);
      
      request.onsuccess = () => {
        const db = request.result;
        try {
          const tx = db.transaction('drafts', 'readonly');
          const store = tx.objectStore('drafts');
          const getAllRequest = store.getAll();
          
          getAllRequest.onsuccess = () => {
            db.close();
            console.log(`[Diagnostics] Exported ${getAllRequest.result.length} drafts`);
            resolve(getAllRequest.result);
          };
          
          getAllRequest.onerror = () => {
            db.close();
            reject(getAllRequest.error);
          };
        } catch (error) {
          db.close();
          reject(error);
        }
      };
      
      request.onupgradeneeded = () => {
        request.transaction?.abort();
        reject(new Error('Database does not exist'));
      };
    });
  }
};

// Make available in browser console for debugging
if (typeof window !== 'undefined') {
  (window as any).dbDiagnostics = dbDiagnostics;
  console.log('[IndexedDB] Diagnostics available: window.dbDiagnostics');
}
