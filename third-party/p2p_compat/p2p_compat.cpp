#include "p2p_compat.h"
#include <memory>

using namespace net;

namespace
{

    class HostHandler: public HostEventHandler {
        public:
            HostHandler() {
                ptr_config = std::make_unique<Config>();
                ptr_host = std::make_unique<Host>(*ptr_config, *this);
            }

            virtual ~HostHandler() = default;

            /*virtual*/
            void OnMessageReceived(const NodeId& from, ByteVector&& message) override;
            /*virtual*/
            void OnNodeDiscovered(const NodeId&) override;
            /*virtual*/
            void OnNodeRemoved(const NodeId&) override;

            static HostHandler& instance();
            void destroy();

        private:
            std::unique_ptr<Config> ptr_config;
            std::unique_ptr<Host> ptr_host;

            static std::unique_ptr<HostHandler> ptr_inst;
    };

    void HostHandler::OnMessageReceived(const NodeId& /*from*/, ByteVector&& /*message*/) {

    }

    void HostHandler::OnNodeDiscovered(const NodeId&) {

    }

    void HostHandler::OnNodeRemoved(const NodeId&) {

    }

    /*static*/
    HostHandler& HostHandler::instance() {
        if(!HostHandler::ptr_inst) {
            HostHandler::ptr_inst = std::make_unique<HostHandler>();
        }
        return *ptr_inst.get();
    }

    void HostHandler::destroy() {
        if(HostHandler::ptr_inst) {
            ptr_inst.reset();
        }
    }

    /*static*/
    std::unique_ptr<HostHandler> HostHandler::ptr_inst;
}

void host_start() {
    HostHandler::instance();
}

void host_stop() {
    HostHandler::instance().destroy();
}
