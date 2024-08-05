//
// Created by Fanchao Liu on 28/6/2024.
//

#include "convo_info_adapters.h"
#include "common_adapters.h"

using namespace nlohmann;

namespace session::config
{
    namespace convo
    {
        NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE_ONLY_SERIALIZE(one_to_one, session_id, last_read, unread);
        NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE_ONLY_SERIALIZE(group, id, last_read, unread);
        NLOHMANN_DEFINE_TYPE_NON_INTRUSIVE_ONLY_SERIALIZE(legacy_group, id, last_read, unread);

        void to_json(json &j, const community &c)
        {
            j["url"] = c.base_url() + "/" + c.room_norm();
            j["pub_key"] = c.pubkey_hex();
            j["last_read"] = c.last_read;
            j["unread"] = c.unread;
        }

        void to_json(json &j, const any &a)
        {
            const char *conv_type;

            if (const auto oto = std::get_if<one_to_one>(&a))
            {
                j = *oto;
                conv_type = "one_to_one";
            }
            else if (const auto g = std::get_if<group>(&a))
            {
                j = *g;
                conv_type = "group";
            }
            else if (const auto comm = std::get_if<community>(&a))
            {
                j = *comm;
                conv_type = "community";
            }
            else if (const auto lg = std::get_if<legacy_group>(&a))
            {
                j = *lg;
                conv_type = "legacy_group";
            }
            else
            {
                return;
            }

            j["type"] = conv_type;
        }
    } // namespace convo

    void to_json(json &out, const ConvoInfoVolatile &c)
    {
        out = json::array();
        for (auto iter = c.begin(); iter != c.end(); ++iter)
        {
            out.push_back(*iter);
        }
    }
}
