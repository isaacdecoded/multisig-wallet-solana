import React, { useState } from "react";
import { Row, Col, Tabs, List, Button, Modal, Divider } from "antd";
import { TransactionForm } from './TransactionForm';
import { TransactionList } from './TransactionList';
import { MultisigWallet } from "../types";

interface Props {
  multisigWallets: MultisigWallet[]
  onCreateTransaction: any
  onApproveTransaction: any
  onExecuteTransaction: any
}

export const MultisigWalletList: React.FC<Props> = ({
  multisigWallets,
  onCreateTransaction,
  onApproveTransaction,
  onExecuteTransaction,
}) => {
  const [isModalOpen, setIsModalOpen] = useState(false);
  if (!multisigWallets || multisigWallets.length <= 0) {
    return (<></>)
  }
  return (
    <Row>
      <Col span={12} offset={4}>
        <Tabs defaultActiveKey="1">
          {
            multisigWallets.map((multisigWallet, idx) => (
              <Tabs.TabPane tab={`Multisig Wallet ${idx+1}`} key={idx}>
                <Row>
                  <Col span={24}>
                    <h3>Threshold: {multisigWallet.threshold.toString()}</h3>
                  </Col>
                  <Col span={24}>
                    <List
                      size="small"
                      header={<div>Owners</div>}
                      bordered
                      dataSource={multisigWallet.owners.map(owner => owner.toString())}
                      renderItem={item => <List.Item>{item}</List.Item>}
                    />
                  </Col>
                  <Divider />
                  <Col span={24}>
                    <Button type="primary" onClick={() => setIsModalOpen(true)}>
                      Create transaction
                    </Button>
                  </Col>
                  <Col span={24}>
                    <TransactionList
                      multisigWallet={multisigWallet}
                      onApprove={onApproveTransaction}
                      onExecute={onExecuteTransaction}
                    />
                  </Col>
                  <Modal
                    title="Create transaction"
                    open={isModalOpen}
                    onCancel={() => setIsModalOpen(false)}
                    footer={null}
                  >
                    <TransactionForm
                      multisigWallet={multisigWallet}
                      onCreate={onCreateTransaction}
                    />
                  </Modal>
                </Row>
              </Tabs.TabPane>
            ))
          }
        </Tabs>
      </Col>
    </Row>
  );
};
