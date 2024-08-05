#include "session_json.h"

#include <iostream>

#include <nlohmann/json.hpp>

#include "user_groups_adapter.h"
#include "user_profile_adapter.h"
#include "contacts_adapter.h"
#include "convo_info_adapters.h"
#include "groups_adapter.h"

#include <session/config/base.hpp>
#include <session/config/contacts.h>
#include <session/config/user_groups.h>
#include <session/config/user_profile.h>
#include <session/config/convo_info_volatile.h>
#include <session/config/groups/info.h>
#include <session/config/groups/members.h>

#include <session/blinding.hpp>

using json = nlohmann::json;

json dump_json(session::config::ConfigBase *config_base)
{
    json out;

    if (const auto user_profile = dynamic_cast<session::config::UserProfile *>(config_base))
    {
        to_json(out, *user_profile);
    }
    else if (const auto convo = dynamic_cast<session::config::ConvoInfoVolatile *>(config_base))
    {
        to_json(out, *convo);
    }
    else if (const auto *user_groups = dynamic_cast<session::config::UserGroups *>(config_base))
    {
        to_json(out, *user_groups);
    }
    else if (const auto contacts = dynamic_cast<session::config::Contacts *>(config_base))
    {
        to_json(out, *contacts);
    }
    else if (const auto *members = dynamic_cast<session::config::groups::Members *>(config_base))
    {
        to_json(out, *members);
    }
    else if (const auto *info = dynamic_cast<session::config::groups::Info *>(config_base))
    {
        to_json(out, *info);
    }

    return out;
}

LIBSESSION_EXPORT const char *session_config_dump_json(const config_object *config)
{
    auto &config_base = session::config::unbox<session::config::ConfigBase>(config);
    auto json = dump_json(config_base.config.get()).dump();
    auto result = malloc(json.length() + 1);
    memcpy(result, json.c_str(), json.length() + 1);
    return static_cast<const char *>(result);
}

LIBSESSION_EXPORT bool session_config_merge(
    config_object *config,
    const unsigned char *data, size_t data_len,
    const char *hash, size_t hash_len,
    char *error_buf, size_t error_buf_len)
{
    session::ustring_view data_view(data, data_len);
    std::string hash_str(hash, hash_len);

    auto &config_base = session::config::unbox<session::config::ConfigBase>(config);

    std::pair<std::string, session::ustring_view> hash_and_data(hash_str, data_view);

    try
    {
        return !config_base.config->merge({hash_and_data}).empty();
    }
    catch (const std::exception &ec)
    {
        if (error_buf)
        {
            strncpy(error_buf, ec.what(), error_buf_len);
        }
        return false;
    }
}

LIBSESSION_EXPORT struct blinded_ids session_create_blind15_id(
    const char *session_id, size_t session_id_len,
    const char *server_pk, size_t server_pk_len)
{
    std::string_view session_id_view(session_id, session_id_len);
    std::string_view server_pk_view(server_pk, server_pk_len);

    try
    {
        auto [id1, id2] = session::blind15_id(session_id_view, server_pk_view);

        struct blinded_ids ids
        {
            .id1 = static_cast<const char *>(malloc(id1.size())),
            .id1_len = id1.size(),
            .id2 = static_cast<const char *>(malloc(id2.size())),
            .id2_len = id2.size()
        };

        memcpy(const_cast<char *>(ids.id1), id1.data(), id1.size());
        memcpy(const_cast<char *>(ids.id2), id2.data(), id2.size());
        return ids;
    }
    catch (const std::exception &ec)
    {
        std::cerr << "Error: " << ec.what() << std::endl;
        return {};
    }
}