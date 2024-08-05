#include "session_json.h"

#include <session/config/contacts.hpp>
#include <session/config/convo_info_volatile.hpp>
#include <session/config/user_profile.hpp>

#include <session/ed25519.hpp>

#include <oxenc/hex.h>
#include <iostream>

using namespace session;

int main(int, char **)
{
    const auto [pubkey, seckey] = ed25519::ed25519_key_pair();

    const auto [person1_pubkey, _] = ed25519::ed25519_key_pair();
    const auto person1_session_id = "05" + oxenc::to_hex(ustring_view(person1_pubkey.data(), person1_pubkey.size()));

    // Contacts
    {
        auto config = std::make_unique<config::Contacts>(ustring_view(seckey.data(), seckey.size()), std::nullopt);
        auto contact = config->get_or_construct(person1_session_id);
        contact.set_name("Test User");
        contact.set_nickname("testuser");
        config->set(contact);

        std::cout << "JSON config: " << session_config_dump_json(config.get()) << std::endl;
    }

    // Conv
    {
        auto config = std::make_unique<config::ConvoInfoVolatile>(
            ustring_view(seckey.data(), seckey.size()), std::nullopt);
        auto c = config->get_or_construct_1to1(person1_session_id);
        c.last_read = std::chrono::system_clock::now().time_since_epoch().count();
        config->set(c);
        assert(config->size_1to1() == 1);

        const auto [group_pubkey, group_seckey] = ed25519::ed25519_key_pair();
        const auto group_pubkey_hex = "03" + oxenc::to_hex(ustring_view(group_pubkey.data(), group_pubkey.size()));
        auto g = config->get_or_construct_group(group_pubkey_hex);
        g.last_read = std::chrono::system_clock::now().time_since_epoch().count();
        config->set(g);
        assert(config->size_groups() == 1);

        std::cout << "JSON config: " << session_config_dump_json(config.get()) << std::endl;
    }

    // User profile
    {
        auto config = std::make_unique<config::UserProfile>(ustring_view(seckey.data(), seckey.size()), std::nullopt);
        config->set_name("Test User");

        std::cout << "JSON config: " << session_config_dump_json(config.get()) << std::endl;
    }

    return 0;
}
