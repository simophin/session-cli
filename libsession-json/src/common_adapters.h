//
// Created by Fanchao Liu on 28/6/2024.
//

#ifndef BASE_CONFIG_ADAPTER_H
#define BASE_CONFIG_ADAPTER_H

#include <nlohmann/json.hpp>
#include <session/config/base.hpp>
#include <session/config/profile_pic.hpp>
#include <session/config/notify.hpp>
#include <session/config/expiring.hpp>

namespace nlohmann
{
    template <typename T>
    struct adl_serializer<std::optional<T>>
    {
        static void to_json(json &j, const std::optional<T> &opt)
        {
            if (!opt.has_value())
            {
                j = nullptr;
            }
            else
            {
                j = *opt; // this will call adl_serializer<T>::to_json which will
                // find the free function to_json in T's namespace!
            }
        }
    };

    template <>
    struct adl_serializer<std::chrono::seconds>
    {
        static void to_json(json &j, const std::chrono::seconds &s)
        {
            j = s.count();
        }
    };
}

namespace session::config
{
    void to_json(nlohmann::json &j, const profile_pic &v);

    NLOHMANN_JSON_SERIALIZE_ENUM(notify_mode,
                                 {
                                     {notify_mode::defaulted, "default"},
                                     {notify_mode::all, "all"},
                                     {notify_mode::disabled, "disabled"},
                                     {notify_mode::mentions_only, "mentions_only"},
                                 });

    NLOHMANN_JSON_SERIALIZE_ENUM(expiration_mode, {
                                                      {expiration_mode::none, "none"},
                                                      {expiration_mode::after_send, "after_send"},
                                                      {expiration_mode::after_read, "after_read"},
                                                  });
}

struct Base64
{
    session::ustring value;
};

void to_json(nlohmann::json &j, const Base64 &v);

#endif // BASE_CONFIG_ADAPTER_H
