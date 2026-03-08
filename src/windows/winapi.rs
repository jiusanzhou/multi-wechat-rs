
pub use winapi::shared::ntdef::{
    HANDLE, 
    NULL, 
    PVOID, 
    ULONG, 
    UNICODE_STRING,
    USHORT,
    NT_SUCCESS,
    LUID,
};

pub use winapi::shared::minwindef::{
    FALSE, 
    TRUE,
    MAX_PATH, 
    DWORD,
    HMODULE,
    HKEY
};

pub use winapi::shared::ntstatus::{
    STATUS_INFO_LENGTH_MISMATCH,
    STATUS_SUCCESS
};

pub use winapi::um::processthreadsapi::{
    GetProcessId,
    GetCurrentProcess,
    OpenProcess,
    OpenProcessToken,
    CreateProcessA,
    CreateProcessW,
};

pub use winapi::um::securitybaseapi::{
    AdjustTokenPrivileges
};

pub use winapi::um::winbase::{
    LookupPrivilegeValueA,
    CREATE_SUSPENDED,
    CREATE_NEW_CONSOLE
};

pub use winapi::um::handleapi::{
    CloseHandle,
    DuplicateHandle,
    INVALID_HANDLE_VALUE
};

pub use winapi::um::tlhelp32::{
    CreateToolhelp32Snapshot,
    Process32Next,
    TH32CS_SNAPPROCESS,
    PROCESSENTRY32,
    MODULEENTRY32,
};

pub use winapi::um::psapi::{
    GetModuleBaseNameW,
    GetModuleFileNameExW
};

pub use winapi::um::winnt::{
    PROCESS_ALL_ACCESS,
    PROCESS_DUP_HANDLE,
    PROCESS_QUERY_LIMITED_INFORMATION,
    DUPLICATE_CLOSE_SOURCE,
    DUPLICATE_SAME_ACCESS,
    MEM_COMMIT,
    MEM_RELEASE,
    MEM_RESERVE,
    PAGE_READWRITE,
    KEY_QUERY_VALUE,
    KEY_READ,
    TOKEN_PRIVILEGES,
    TOKEN_ADJUST_PRIVILEGES,
    TOKEN_QUERY,
    SE_PRIVILEGE_ENABLED,
    SE_DEBUG_NAME,
    LUID_AND_ATTRIBUTES
};

pub use winapi::um::memoryapi::{
    ReadProcessMemory,
    WriteProcessMemory,
    VirtualAllocEx,
    VirtualFreeEx,
};

pub use winapi::um::winreg::{
    RegOpenKeyW,
    RegOpenKeyA,
    RegOpenKeyExA,
    RegOpenKeyExW,
    RegQueryValueA,
    RegQueryValueW,
    RegQueryValueExA,
    RegQueryValueExW,
    RegCloseKey,
    HKEY_CURRENT_USER,
};

pub use winapi::um::winuser::{
    MessageBoxA,
    MessageBoxW,
    MB_OK
};

pub use ntapi::ntzwapi::{
    ZwQuerySystemInformation,
};

pub use ntapi::ntexapi::{
    SystemHandleInformation,
    SYSTEM_HANDLE_INFORMATION,
    SYSTEM_HANDLE_TABLE_ENTRY_INFO,
    NtQuerySystemInformation
};

pub use ntapi::ntobapi::{
    ObjectNameInformation,
    ObjectTypeInformation,
    OBJECT_NAME_INFORMATION,
    OBJECT_TYPE_INFORMATION,
    OBJECT_INFORMATION_CLASS,
    NtQueryObject
};