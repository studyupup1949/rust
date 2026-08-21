# aam-cConfig.cmake

set(AAM_C_CMAKE_DIR "${CMAKE_CURRENT_LIST_DIR}")

get_filename_component(AAM_C_INSTALL_PREFIX "${AAM_C_CMAKE_DIR}/../../.." ABSOLUTE)

set(AAM_C_INCLUDE_DIRS "${AAM_C_INSTALL_PREFIX}/include")

if(NOT TARGET aam-c::aam-c)
    add_library(aam-c::aam-c SHARED IMPORTED)

    if(WIN32)
        # Windows
        set_target_properties(aam-c::aam-c PROPERTIES
            IMPORTED_LOCATION "${AAM_C_INSTALL_PREFIX}/bin/aam_c.dll"
            IMPORTED_IMPLIB "${AAM_C_INSTALL_PREFIX}/lib/aam_c.lib"
            INTERFACE_INCLUDE_DIRECTORIES "${AAM_C_INCLUDE_DIRS}"
        )
    elseif(APPLE)
        # macOS
        set_target_properties(aam-c::aam-c PROPERTIES
            IMPORTED_LOCATION "${AAM_C_INSTALL_PREFIX}/lib/libaam_c.dylib"
            INTERFACE_INCLUDE_DIRECTORIES "${AAM_C_INCLUDE_DIRS}"
        )
    else()
        # Linux
        set_target_properties(aam-c::aam-c PROPERTIES
            IMPORTED_LOCATION "${AAM_C_INSTALL_PREFIX}/lib/libaam_c.so"
            INTERFACE_INCLUDE_DIRECTORIES "${AAM_C_INCLUDE_DIRS}"
        )
    endif()
endif()