#pragma once

#include <session/config/base.h>

#ifdef __cplusplus
extern "C"
{
#endif
    /**
     * @return the JSON string of the session config. Must be freed by the caller.
     */
    LIBSESSION_EXPORT const char *session_config_dump_json(const config_object *config);

    LIBSESSION_EXPORT bool session_config_merge(config_object *config,
                                                const unsigned char *data, size_t data_len,
                                                const char *hash, size_t hash_len,
                                                char *error_buf, size_t error_buf_len);

    struct blinded_ids
    {
        const char *id1;
        size_t id1_len;

        const char *id2;
        size_t id2_len;
    };

    LIBSESSION_EXPORT struct blinded_ids
    session_create_blind15_id(
        const char *session_id, size_t session_id_len,
        const char *server_pk, size_t server_pk_len);

#ifdef __cplusplus
}
#endif
