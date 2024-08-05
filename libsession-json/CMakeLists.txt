
add_library(
        session-json
        STATIC
        src/session_json.cpp
        src/session_json.h
        src/user_profile_adapter.cpp
        src/user_profile_adapter.h
        src/common_adapters.cpp
        src/common_adapters.h
        src/user_groups_adapter.cpp
        src/user_groups_adapter.h
        src/contacts_adapter.cpp
        src/contacts_adapter.h
        src/convo_info_adapters.cpp
        src/convo_info_adapters.h
        src/groups_adapter.cpp
        src/groups_adapter.h
)

target_link_libraries(
        session-json
        PUBLIC
        config
        oxenc
        quic
        util
        nlohmann_json::nlohmann_json
        fmt
)

add_executable(
        session-json-test
        src/test.cpp
)

target_link_libraries(
        session-json-test
        session-json
        crypto
)

set_target_properties(
        session-json session-json-test
        PROPERTIES
        CXX_STANDARD 20
        CXX_STANDARD_REQUIRED true
)