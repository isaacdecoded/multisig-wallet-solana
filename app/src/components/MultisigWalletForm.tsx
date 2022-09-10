import React, { useState } from 'react';
import { Button, Form, Input } from "antd";
import { MinusCircleOutlined, PlusOutlined } from '@ant-design/icons';

const formItemLayout = {
  labelCol: {
    xs: { span: 24 },
    sm: { span: 4 },
  },
  wrapperCol: {
    xs: { span: 24 },
    sm: { span: 20 },
  },
};
const formItemLayoutWithOutLabel = {
  wrapperCol: {
    xs: { span: 24, offset: 0 },
    sm: { span: 20, offset: 4 },
  },
};

interface Props {
  onCreate: any
  addOwner: any
}

export const MultisigWalletForm: React.FC<Props> = ({
  onCreate,
  addOwner,
}) => {
  const [form] = Form.useForm();
  const [loading, setLoading] = useState(false)
  return (
    <Form
      form={form}
      {...formItemLayoutWithOutLabel}
      onFinish={async (values) => {
        setLoading(true)
        await onCreate(values);
        form.resetFields();
        setLoading(false)
      }}
    >
      <Form.Item
        name="threshold"
        label="Threshold"
        rules={[
          { required: true },
        ]}
        {...formItemLayout}
      >
        <Input placeholder="Threshold" style={{ width: '60%' }} />
      </Form.Item>
      <Form.List
        name="ownerAddresses"
        rules={[
          {
            validator: async (_, ownerAddresses) => {
              if (!ownerAddresses || ownerAddresses.length < 1) {
                Promise.reject(new Error('At least 1 owner address'));
              }
            },
          },
        ]}
      >
        {(fields, { add, remove }, { errors }) => (
          <>
            {fields.map((field, index) => (
              <Form.Item
                {...(index === 0 ? formItemLayout : formItemLayoutWithOutLabel)}
                label={index === 0 ? 'Owners' : ''}
                required={false}
                key={field.key}
              >
                <Form.Item
                  {...field}
                  noStyle
                >
                  <Input placeholder="Owner's address" style={{ width: '60%' }} />
                </Form.Item>
                {fields.length > 1 ? (
                  <MinusCircleOutlined
                    className="dynamic-delete-button"
                    onClick={() => remove(field.name)}
                  />
                ) : null}
              </Form.Item>
            ))}
            <Form.Item>
              <Button
                type="dashed"
                onClick={() => {
                  const value = addOwner()
                  add(value)
                }}
                style={{ width: '60%' }}
                icon={<PlusOutlined />}
              >
                Add owner
              </Button>
              <Form.ErrorList errors={errors} />
            </Form.Item>
          </>
        )}
      </Form.List>
      <Form.Item>
        <Button type="primary" htmlType="submit" loading={loading}>
          Create
        </Button>
      </Form.Item>
    </Form>
  );
};
